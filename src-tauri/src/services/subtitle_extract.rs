use std::fmt::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Hard cap so ffmpeg can't run forever on a corrupt/huge file. The user
/// gets an actionable error instead of an indefinite spinner.
const FFMPEG_TIMEOUT: Duration = Duration::from_secs(60);

/// Probe limits passed to every ffmpeg invocation. Default analyze duration
/// (~5s of stream) is enough for subtitle stream detection but small enough
/// that 4K H.265 files don't cost minutes of header scanning.
const PROBE_FLAGS: &[&str] = &["-probesize", "50M", "-analyzeduration", "5000000"];

fn cmd(program: &str) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

/// Run ffmpeg with our timeout + probe limits, killing the child process on
/// timeout so we don't leak a runaway extractor. Returns stdout bytes.
async fn run_ffmpeg(args: &[&str]) -> Result<Vec<u8>, String> {
    let ffmpeg = find_ffmpeg().ok_or("ffmpeg not found")?;
    let mut child = cmd(&ffmpeg)
        .args(PROBE_FLAGS)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("ffmpeg spawn failed: {e}"))?;

    let mut stdout = child.stdout.take().ok_or("no stdout")?;
    let mut buf = Vec::new();
    let read_fut = async {
        stdout.read_to_end(&mut buf).await.map_err(|e| format!("read failed: {e}"))?;
        let _ = child.wait().await;
        Ok::<Vec<u8>, String>(buf)
    };
    match tokio::time::timeout(FFMPEG_TIMEOUT, read_fut).await {
        Ok(res) => res,
        Err(_) => {
            let _ = child.kill().await;
            Err("ffmpeg timed out — file may be too large or codec unsupported".into())
        }
    }
}

#[derive(Clone)]
pub struct SubEntry {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub style: String,
}

/// Extract embedded subtitles via ffmpeg. Uses ASS format if source is ASS, SRT otherwise.
/// mpv numbers subtitle tracks by type starting at 1; ffmpeg's `0:s:N`
/// indexes the same set starting at 0, so the mapping is just `sid - 1`.
/// Skipping the previous "list streams to confirm" pre-pass cuts one full
/// ffmpeg header scan per call — a big win on large H.265 files.
pub async fn extract_from_video(path: &str, mpv_track_id: Option<i64>, is_ass: bool) -> Result<Vec<SubEntry>, String> {
    let stream = format!("0:s:{}", mpv_track_id.unwrap_or(1).saturating_sub(1));
    let fmt = if is_ass { "ass" } else { "srt" };
    let bytes = run_ffmpeg(&["-i", path, "-map", &stream, "-f", fmt, "-"]).await?;
    let text = String::from_utf8_lossy(&bytes);
    if text.trim().is_empty() { return Err("No subtitle data extracted".into()); }
    let content = text.replace("\r\n", "\n");
    if is_ass { parse_ass_content(&content) } else { parse_srt_content(&content) }
}

fn parse_ass_content(content: &str) -> Result<Vec<SubEntry>, String> {
    let mut entries = Vec::new();
    for line in content.lines() {
        if !line.starts_with("Dialogue:") { continue; }
        let parts: Vec<&str> = line["Dialogue:".len()..].trim_start().splitn(10, ',').collect();
        if parts.len() < 10 { continue; }
        let text = strip_tags(parts[9].trim());
        if !text.is_empty() {
            entries.push(SubEntry { start_ms: ass_time_to_ms(parts[1].trim()), end_ms: ass_time_to_ms(parts[2].trim()), text, style: parts[3].trim().to_string() });
        }
    }
    entries.sort_by_key(|e| e.start_ms);
    Ok(entries)
}

/// Extract ASS header from embedded track via ffmpeg.
pub async fn extract_ass_header_from_video(path: &str, mpv_track_id: i64) -> Result<String, String> {
    let stream = format!("0:s:{}", mpv_track_id.saturating_sub(1));
    let bytes = run_ffmpeg(&["-i", path, "-map", &stream, "-f", "ass", "-"]).await?;
    extract_header_from_ass_content(&String::from_utf8_lossy(&bytes))
}

/// Extract ASS header from external .ass file.
pub fn extract_ass_header_from_file(path: &str) -> Result<String, String> {
    extract_header_from_ass_content(&read_text_file(path)?)
}

/// Parse external SRT file.
pub fn extract_from_srt(path: &str) -> Result<Vec<SubEntry>, String> {
    parse_srt_content(&read_text_file(path)?.replace("\r\n", "\n"))
}

/// Parse external ASS/SSA file.
pub fn extract_from_ass(path: &str) -> Result<Vec<SubEntry>, String> {
    let content = read_text_file(path)?.replace("\r\n", "\n");
    let mut entries = Vec::new();
    for line in content.lines() {
        if !line.starts_with("Dialogue:") { continue; }
        let parts: Vec<&str> = line["Dialogue:".len()..].trim_start().splitn(10, ',').collect();
        if parts.len() < 10 { continue; }
        let text = strip_tags(parts[9].trim());
        if !text.is_empty() {
            entries.push(SubEntry { start_ms: ass_time_to_ms(parts[1].trim()), end_ms: ass_time_to_ms(parts[2].trim()), text, style: parts[3].trim().to_string() });
        }
    }
    entries.sort_by_key(|e| e.start_ms);
    Ok(entries)
}

/// Write translated entries as ASS (preserving original header/styles).
pub fn write_ass(entries: &[SubEntry], header: &str, path: &str) -> Result<(), String> {
    let mut out = String::with_capacity(header.len() + entries.len() * 100);
    out.push_str(header);
    for e in entries {
        let _ = writeln!(out, "Dialogue: 0,{},{},{},,0,0,0,,{}", ms_to_ass(e.start_ms), ms_to_ass(e.end_ms), e.style, e.text.replace('\n', "\\N"));
    }
    std::fs::write(path, out).map_err(|e| format!("Cannot write: {e}"))
}

/// Write translated entries as SRT.
pub fn write_srt(entries: &[SubEntry], path: &str) -> Result<(), String> {
    let mut out = String::with_capacity(entries.len() * 80);
    for (i, e) in entries.iter().enumerate() {
        let _ = writeln!(out, "{}\n{} --> {}\n{}\n", i + 1, ms_to_srt(e.start_ms), ms_to_srt(e.end_ms), e.text);
    }
    std::fs::write(path, out).map_err(|e| format!("Cannot write: {e}"))
}

// --- Internal ---

fn extract_header_from_ass_content(content: &str) -> Result<String, String> {
    let mut header = String::new();
    for line in content.lines() {
        header.push_str(line);
        header.push('\n');
        if line.trim_start().starts_with("Format:") && header.contains("[Events]") { break; }
    }
    if header.is_empty() || !header.contains("[Events]") { return Err("No ASS header found".into()); }
    Ok(header)
}

fn read_text_file(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("Cannot read: {e}"))?;
    let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) { &bytes[3..] } else { &bytes };
    match std::str::from_utf8(bytes) {
        Ok(s) => Ok(s.to_string()),
        Err(_) => Ok(bytes.iter().map(|&b| b as char).collect()),
    }
}

fn strip_tags(text: &str) -> String {
    // Skip drawing commands (vector shapes rendered as numbers)
    if text.contains("\\p1") || text.contains("\\p2") || text.contains("\\p3") {
        return String::new();
    }
    let mut out = String::with_capacity(text.len());
    let (mut in_ass, mut in_html) = (false, false);
    for c in text.chars() {
        match c {
            '{' => in_ass = true, '}' => in_ass = false,
            '<' => in_html = true, '>' => { in_html = false; continue; }
            _ if !in_ass && !in_html => out.push(c), _ => {}
        }
    }
    out.replace("\\N", "\n").replace("\\n", "\n")
}

fn parse_srt_content(content: &str) -> Result<Vec<SubEntry>, String> {
    let mut entries = Vec::new();
    for block in content.split("\n\n") {
        let lines: Vec<&str> = block.trim().lines().collect();
        if lines.len() < 3 { continue; }
        let Some((start, end)) = parse_srt_timing(lines[1]) else { continue };
        let text = strip_tags(&lines[2..].join("\n")).trim().to_string();
        if !text.is_empty() { entries.push(SubEntry { start_ms: start, end_ms: end, text, style: "Default".into() }); }
    }
    Ok(entries)
}

fn parse_srt_timing(line: &str) -> Option<(u64, u64)> {
    let (a, b) = line.split_once("-->")?;
    Some((srt_time_to_ms(a.trim())?, srt_time_to_ms(b.trim())?))
}

fn srt_time_to_ms(t: &str) -> Option<u64> {
    let t = t.replace(',', ".");
    let p: Vec<&str> = t.split(':').collect();
    if p.len() != 3 { return None; }
    let h: u64 = p[0].parse().ok()?;
    let m: u64 = p[1].parse().ok()?;
    let (s, ms): (u64, u64) = match p[2].split_once('.') {
        Some((s, f)) => (s.parse().ok()?, format!("{:0<3}", &f[..f.len().min(3)]).parse().unwrap_or(0)),
        None => (p[2].parse().ok()?, 0),
    };
    Some(h * 3600000 + m * 60000 + s * 1000 + ms)
}

fn ass_time_to_ms(t: &str) -> u64 {
    let p: Vec<&str> = t.split(':').collect();
    if p.len() != 3 { return 0; }
    let h: u64 = p[0].parse().unwrap_or(0);
    let m: u64 = p[1].parse().unwrap_or(0);
    let (s, cs) = match p[2].split_once('.') {
        Some((s, f)) => (s.parse().unwrap_or(0u64), f.parse().unwrap_or(0u64)),
        None => (p[2].parse().unwrap_or(0), 0),
    };
    h * 3600000 + m * 60000 + s * 1000 + cs * 10
}

fn ms_to_ass(ms: u64) -> String {
    format!("{}:{:02}:{:02}.{:02}", ms / 3600000, (ms % 3600000) / 60000, (ms % 60000) / 1000, (ms % 1000) / 10)
}

fn ms_to_srt(ms: u64) -> String {
    format!("{:02}:{:02}:{:02},{:03}", ms / 3600000, (ms % 3600000) / 60000, (ms % 60000) / 1000, ms % 1000)
}

fn find_ffmpeg() -> Option<String> {
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        for sub in ["", "binaries"] {
            let p = dir.join(sub).join("ffmpeg.exe");
            if p.exists() { return Some(p.to_string_lossy().into()); }
        }
    }
    if ffmpeg_on_path() { return Some("ffmpeg".into()); }
    for base in ["C:/ffmpeg/bin", "C:/Program Files/ffmpeg/bin"] {
        let p = format!("{base}/ffmpeg.exe");
        if Path::new(&p).exists() { return Some(p); }
    }
    None
}

fn ffmpeg_on_path() -> bool {
    let mut c = std::process::Command::new("ffmpeg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.arg("-version").output().is_ok()
}
