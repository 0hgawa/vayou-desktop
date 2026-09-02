use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use flate2::read::GzDecoder;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://rest.opensubtitles.org/search";
const USER_AGENT: &str = "Vayou v1.0";
const TIMEOUT: Duration = Duration::from_secs(15);
const HASH_CHUNK: u64 = 65536;
const MAX_RESULTS: usize = 50;

/// Subtitle search result. Serialized as snake_case for the frontend; accepts
/// the REST .org MixedCase keys as deserialization aliases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResult {
    #[serde(default, alias = "SubFileName")]
    pub name: String,
    #[serde(default, alias = "SubLanguageID")]
    pub lang: String,
    #[serde(default, alias = "SubDownloadLink")]
    pub download_link: String,
    #[serde(default, alias = "SubDownloadsCnt")]
    pub downloads: String,
    #[serde(default, alias = "MatchedBy")]
    pub matched_by: String,
}

fn http() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(USER_AGENT)
            .timeout(TIMEOUT)
            .connect_timeout(TIMEOUT)
            .build()
            .expect("reqwest client")
    })
}

async fn perform_search(path_params: &str) -> Result<Vec<SubResult>, String> {
    let url = format!("{BASE_URL}/{path_params}");
    let resp = http()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Search failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Search failed: HTTP {}", resp.status()));
    }
    resp.json::<Vec<SubResult>>()
        .await
        .map_err(|e| format!("Parse error: {e}"))
}

async fn search_by_hash(
    movie_hash: &str,
    movie_byte_size: u64,
    lang: &str,
) -> Result<Vec<SubResult>, String> {
    let mut params = format!("moviebytesize-{movie_byte_size}/moviehash-{movie_hash}");
    if !lang.is_empty() {
        params.push_str("/sublanguageid-");
        params.push_str(lang);
    }
    perform_search(&params).await
}

async fn search_by_query_once(query: &str, lang: &str) -> Result<Vec<SubResult>, String> {
    let encoded = urlencoding::encode(query).replace('+', "%20");
    let mut params = format!("query-{encoded}");
    if !lang.is_empty() {
        params.push_str("/sublanguageid-");
        params.push_str(lang);
    }
    perform_search(&params).await
}

/// Search OpenSubtitles by free-text query. The legacy REST endpoint matches
/// inconsistently across case — the same title can return dozens of hits in
/// one casing and zero in another — so when the original query yields
/// nothing we re-run it lowercased before giving up. Costs an extra request
/// only on a miss; no local normalization that could hide legitimate hits.
async fn search_by_query(query: &str, lang: &str) -> Result<Vec<SubResult>, String> {
    let primary = search_by_query_once(query, lang).await.unwrap_or_default();
    if !primary.is_empty() {
        return Ok(primary);
    }
    let lower = query.to_lowercase();
    if lower != query {
        return Ok(search_by_query_once(&lower, lang).await.unwrap_or_default());
    }
    Ok(primary)
}

/// Combined search: hash (when available) + query, deduped by download link, capped at 50.
pub async fn search(
    file_hash: Option<(String, u64)>,
    query: &str,
    lang: &str,
) -> Result<Vec<SubResult>, String> {
    let hash_results = match file_hash {
        Some((hash, size)) => search_by_hash(&hash, size, lang).await.unwrap_or_default(),
        None => Vec::new(),
    };
    let query_results = if query.is_empty() {
        Vec::new()
    } else {
        search_by_query(query, lang).await.unwrap_or_default()
    };

    let mut seen = std::collections::HashSet::new();
    let merged: Vec<SubResult> = hash_results
        .into_iter()
        .chain(query_results)
        .filter(|r| !r.download_link.is_empty() && seen.insert(r.download_link.clone()))
        .take(MAX_RESULTS)
        .collect();
    Ok(merged)
}

/// Download and gunzip a subtitle into `dir/safe_name`. Returns the saved path.
pub async fn download(
    download_link: &str,
    dir: &Path,
    file_name: &str,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("mkdir failed: {e}"))?;

    let safe_name = Path::new(file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("subtitle.srt");
    let target = dir.join(safe_name);

    let bytes = http()
        .get(download_link)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Read failed: {e}"))?;

    let mut decoder = GzDecoder::new(bytes.as_ref());
    let mut out = File::create(&target).map_err(|e| format!("Write failed: {e}"))?;
    let mut buf = [0u8; 8192];
    loop {
        let n = decoder
            .read(&mut buf)
            .map_err(|e| format!("Gunzip failed: {e}"))?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])
            .map_err(|e| format!("Write failed: {e}"))?;
    }
    Ok(target)
}

/// OpenSubtitles file hash: 64-bit sum of file size + 8-byte LE longs across
/// the first and last 64 KiB. Mirrors `OpenSubtitlesHasher.kt`.
pub fn compute_hash(path: &str) -> Result<(String, u64), String> {
    let mut file = std::fs::File::open(path).map_err(|e| format!("Cannot open: {e}"))?;
    let size = file
        .metadata()
        .map_err(|e| format!("Cannot read size: {e}"))?
        .len();
    if size < HASH_CHUNK {
        return Err("File too small".into());
    }

    let mut hash: u64 = size;
    let mut buf = vec![0u8; HASH_CHUNK as usize];

    file.read_exact(&mut buf)
        .map_err(|e| format!("Read error: {e}"))?;
    // `as_chunks::<8>()` yields `[u8; 8]` directly, so the length is proven at
    // compile time — no `try_into().unwrap()` on a slice that can only ever be
    // eight bytes long. `.0` drops the remainder, which is always empty here
    // because the buffer is a multiple of 8.
    for chunk in buf.as_chunks::<8>().0 {
        hash = hash.wrapping_add(u64::from_le_bytes(*chunk));
    }

    file.seek(SeekFrom::End(-(HASH_CHUNK as i64)))
        .map_err(|e| format!("Seek error: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("Read error: {e}"))?;
    for chunk in buf.as_chunks::<8>().0 {
        hash = hash.wrapping_add(u64::from_le_bytes(*chunk));
    }

    Ok((format!("{hash:016x}"), size))
}

#[cfg(test)]
mod tests {
    use super::compute_hash;

    /// Writes `bytes` to a uniquely named temp file and hands back the path.
    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("vayou-hash-{name}.bin"));
        std::fs::write(&path, bytes).expect("write fixture");
        path
    }

    #[test]
    fn hash_seeds_from_the_file_size() {
        // 128 KiB of zeros: neither 64 KiB window contributes anything, so the
        // hash is the file size alone — a value derivable without running the
        // implementation. 131072 == 0x20000.
        let path = temp_file("zeros", &vec![0u8; 128 * 1024]);
        let (hash, size) = compute_hash(path.to_str().unwrap()).unwrap();
        assert_eq!(size, 131_072);
        assert_eq!(hash, "0000000000020000");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn every_eight_byte_word_is_summed_little_endian() {
        // One byte set to 1 at the very start: the first window reads it as the
        // low byte of a little-endian u64 and adds 1. The trailing window is
        // all zeros. 131072 + 1 == 0x20001.
        let mut bytes = vec![0u8; 128 * 1024];
        bytes[0] = 1;
        let path = temp_file("one", &bytes);
        let (hash, _) = compute_hash(path.to_str().unwrap()).unwrap();
        assert_eq!(hash, "0000000000020001");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_two_windows_overlap_on_a_single_chunk_file() {
        // At exactly 64 KiB the head and tail windows are the same bytes, so a
        // set byte is counted twice. 65536 + 1 + 1 == 0x10002. Guards the
        // seek-from-end arithmetic against an off-by-one.
        let mut bytes = vec![0u8; 64 * 1024];
        bytes[0] = 1;
        let path = temp_file("overlap", &bytes);
        let (hash, size) = compute_hash(path.to_str().unwrap()).unwrap();
        assert_eq!(size, 65_536);
        assert_eq!(hash, "0000000000010002");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_a_file_shorter_than_one_window() {
        // Too small to read a 64 KiB window: refused up front rather than
        // hashing whatever happens to be in the buffer.
        let path = temp_file("tiny", b"not enough bytes");
        assert!(compute_hash(path.to_str().unwrap()).is_err());
        let _ = std::fs::remove_file(&path);
    }
}
