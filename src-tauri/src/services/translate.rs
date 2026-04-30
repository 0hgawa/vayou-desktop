use std::collections::HashMap;
use std::sync::Mutex;

static CACHE: std::sync::OnceLock<Mutex<HashMap<(String, String), String>>> =
    std::sync::OnceLock::new();

fn cache() -> &'static Mutex<HashMap<(String, String), String>> {
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Translate a block of text (can contain \n) in a SINGLE POST request.
pub async fn translate(text: &str, target_lang: &str) -> Result<String, String> {
    let text = text.trim().to_string();
    if text.is_empty() { return Ok(String::new()); }

    let key = (text.clone(), target_lang.to_string());
    if let Ok(c) = cache().lock() {
        if let Some(cached) = c.get(&key) { return Ok(cached.clone()); }
    }

    let client = reqwest::Client::new();
    let resp = client
        .post("https://translate.googleapis.com/translate_a/single")
        .query(&[("client", "gtx"), ("sl", "auto"), ("tl", target_lang), ("dt", "t")])
        .form(&[("q", text.as_str())])
        .send().await.map_err(|e| format!("HTTP error: {e}"))?;

    if resp.status() == 429 || resp.status() == 403 {
        return Err("Rate limited".into());
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("JSON error: {e}"))?;
    let mut result = String::new();
    if let Some(segs) = body.get(0).and_then(|v| v.as_array()) {
        for seg in segs {
            if let Some(t) = seg.get(0).and_then(|v| v.as_str()) { result.push_str(t); }
        }
    }
    if result.is_empty() { return Err("Empty response".into()); }

    if let Ok(mut c) = cache().lock() { c.insert(key, result.clone()); }
    Ok(result)
}
