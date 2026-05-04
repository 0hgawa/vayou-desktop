use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use serde_json::Value;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                  AppleWebKit/537.36 (KHTML, like Gecko) \
                  Chrome/131.0.0.0 Safari/537.36";

static CACHE: OnceLock<Mutex<HashMap<(String, String), String>>> = OnceLock::new();
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<(String, String), String>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(UA)
            .timeout(Duration::from_secs(15))
            .build()
            .expect("translate http client")
    })
}

/// Translate `text` into `target_lang`. Tries the Chrome-extension endpoint
/// (`clients5.google.com/dict-chrome-ex`) first because Google rate-limits
/// it far less than the public `gtx` endpoint, then falls back to `gtx` if
/// the first one fails. Each endpoint retries up to 3 times with backoff
/// when it hits 429/403.
pub async fn translate(text: &str, target_lang: &str) -> Result<String, String> {
    let text = text.trim().to_string();
    if text.is_empty() { return Ok(String::new()); }

    let key = (text.clone(), target_lang.to_string());
    if let Ok(c) = cache().lock() {
        if let Some(v) = c.get(&key) { return Ok(v.clone()); }
    }

    let result = match try_chrome_ext(&text, target_lang).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = %e, "translate: chrome-ext failed, falling back to gtx");
            try_gtx(&text, target_lang).await?
        }
    };

    if let Ok(mut c) = cache().lock() { c.insert(key, result.clone()); }
    Ok(result)
}

async fn try_chrome_ext(text: &str, tl: &str) -> Result<String, String> {
    let resp = with_backoff(|| client()
        .get("https://clients5.google.com/translate_a/t")
        .query(&[("client", "dict-chrome-ex"), ("sl", "auto"), ("tl", tl), ("q", text)])
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
    ).await?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    // The endpoint returns either a flat `["translated"]` for a single q,
    // or `[["seg1", "seg2", ...]]` when q has line breaks.
    if let Some(s) = body.get(0).and_then(|v| v.as_str()) {
        return Ok(s.to_string());
    }
    if let Some(arr) = body.get(0).and_then(|v| v.as_array()) {
        let s: String = arr.iter().filter_map(|v| v.as_str()).collect();
        if !s.is_empty() { return Ok(s); }
    }
    Err("empty chrome-ext response".into())
}

async fn try_gtx(text: &str, tl: &str) -> Result<String, String> {
    let resp = with_backoff(|| client()
        .get("https://translate.googleapis.com/translate_a/single")
        .query(&[("client", "gtx"), ("sl", "auto"), ("tl", tl), ("dt", "t"), ("q", text)])
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
    ).await?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut out = String::new();
    if let Some(segs) = body.get(0).and_then(|v| v.as_array()) {
        for seg in segs {
            if let Some(t) = seg.get(0).and_then(|v| v.as_str()) { out.push_str(t); }
        }
    }
    if out.is_empty() { Err("empty gtx response".into()) } else { Ok(out) }
}

/// Sends a request, retrying up to 3 times with exponential backoff
/// (immediate, +1s, +2s) when the server replies 429 or 403. Any other
/// error or non-success status returns immediately — only rate limits
/// are worth waiting on.
async fn with_backoff<F, Fut>(mut send: F) -> Result<reqwest::Response, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = reqwest::Result<reqwest::Response>>,
{
    let delays_ms = [0u64, 1000, 2000];
    let mut last_status = None;
    for delay in delays_ms {
        if delay > 0 { tokio::time::sleep(Duration::from_millis(delay)).await; }
        match send().await {
            Ok(r) if matches!(r.status().as_u16(), 429 | 403) => {
                last_status = Some(r.status());
            }
            Ok(r) if !r.status().is_success() => return Err(format!("HTTP {}", r.status())),
            Ok(r) => return Ok(r),
            Err(e) => return Err(e.to_string()),
        }
    }
    Err(format!("rate-limited (HTTP {})", last_status.unwrap()))
}
