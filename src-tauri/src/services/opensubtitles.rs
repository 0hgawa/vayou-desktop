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
        params.push_str(&format!("/sublanguageid-{lang}"));
    }
    perform_search(&params).await
}

async fn search_by_query_once(query: &str, lang: &str) -> Result<Vec<SubResult>, String> {
    let encoded = urlencoding::encode(query).replace('+', "%20");
    let mut params = format!("query-{encoded}");
    if !lang.is_empty() {
        params.push_str(&format!("/sublanguageid-{lang}"));
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
    let query_results = if !query.is_empty() {
        search_by_query(query, lang).await.unwrap_or_default()
    } else {
        Vec::new()
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
    let mut buf = [0u8; HASH_CHUNK as usize];

    file.read_exact(&mut buf)
        .map_err(|e| format!("Read error: {e}"))?;
    for chunk in buf.chunks_exact(8) {
        hash = hash.wrapping_add(u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    file.seek(SeekFrom::End(-(HASH_CHUNK as i64)))
        .map_err(|e| format!("Seek error: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("Read error: {e}"))?;
    for chunk in buf.chunks_exact(8) {
        hash = hash.wrapping_add(u64::from_le_bytes(chunk.try_into().unwrap()));
    }

    Ok((format!("{:016x}", hash), size))
}
