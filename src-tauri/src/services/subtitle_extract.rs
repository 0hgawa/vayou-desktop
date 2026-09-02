use std::fmt::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

/// `CREATE_NO_WINDOW` flag for `Command::creation_flags` — keeps ffmpeg from
/// flashing a console window when invoked from a windowed app. Windows-only;
/// the corresponding `CommandExt` import is also Windows-gated below.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// `BELOW_NORMAL_PRIORITY_CLASS`. An extraction reads the whole container as
/// fast as the disk allows, and it does that while mpv is decoding from the
/// same disk — on a 22 GB UHD remux the two compete and playback stutters for
/// as long as the extraction runs. Demoting ffmpeg lets the player win that
/// contest. The extraction is background work nobody is watching frame by
/// frame, so it can afford to lose.
#[cfg(windows)]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;

/// How long an extraction may take, as a function of the file it reads.
///
/// ffmpeg has to demux the whole container to collect an embedded subtitle
/// track, so the wall time scales with the size of the file, not with how much
/// text the track holds — a fixed cap was the wrong shape. A flat 30s failed
/// every large movie: a 10.8 GB WEBRip measures 50s to extract a 2.5 KB forced
/// track.
///
/// The budget is 50 MB/s, comfortably under a warm local disk, so a slow or
/// network drive still fits. The floor keeps small files failing fast when the
/// track index is wrong, and the ceiling means a pathological file still ends
/// in an error rather than an indefinite spinner.
fn timeout_for_size(bytes: u64) -> Duration {
    const FLOOR_S: u64 = 30;
    const CEILING_S: u64 = 600;
    const BYTES_PER_SECOND: u64 = 50 * 1024 * 1024;
    Duration::from_secs((bytes / BYTES_PER_SECOND).clamp(FLOOR_S, CEILING_S))
}

/// The size of the file ffmpeg is about to read, or 0 when it cannot be read —
/// which lands on the floor above, the same budget the flat cap used to give.
fn source_size(path: &str) -> u64 {
    std::fs::metadata(path).map_or(0, |m| m.len())
}

/// Probe limits passed to every ffmpeg invocation. Default analyze duration
/// (~5s of stream) is enough for subtitle stream detection but small enough
/// that 4K H.265 files don't cost minutes of header scanning.
const PROBE_FLAGS: &[&str] = &["-probesize", "50M", "-analyzeduration", "5000000"];

fn cmd(program: &str) -> Command {
    let mut c = Command::new(program);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
    c
}

/// Run ffmpeg with our timeout + probe limits, killing the child process on
/// timeout so we don't leak a runaway extractor. Returns stdout bytes.
async fn run_ffmpeg(args: &[&str], source: &str) -> Result<Vec<u8>, String> {
    let ffmpeg = find_ffmpeg().ok_or("ffmpeg not found")?;
    let started = std::time::Instant::now();
    tracing::info!(args = ?args, "ffmpeg: starting");
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
    match tokio::time::timeout(timeout_for_size(source_size(source)), read_fut).await {
        Ok(Ok(buf)) => {
            tracing::info!(elapsed_ms = started.elapsed().as_millis() as u64, bytes = buf.len(), "ffmpeg: done");
            Ok(buf)
        }
        Ok(Err(e)) => {
            tracing::warn!(elapsed_ms = started.elapsed().as_millis() as u64, error = %e, "ffmpeg: read error");
            Err(e)
        }
        Err(_) => {
            let _ = child.kill().await;
            tracing::warn!(elapsed_ms = started.elapsed().as_millis() as u64, "ffmpeg: timed out");
            Err("ffmpeg timed out — the file may be unreadable or the subtitle codec unsupported".into())
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

/// An embedded text subtitle track, as SRT.
///
/// mpv numbers subtitle tracks by type starting at 1; ffmpeg's `0:s:N` indexes
/// the same set starting at 0, so the mapping is just `sid - 1`. There is no
/// "list the streams first" pre-pass: that cost one extra full header scan per
/// call, which is expensive on large H.265 files.
///
/// ASS goes through `extract_ass_from_video`, which also needs the header.
pub async fn extract_srt_from_video(path: &str, mpv_track_id: i64) -> Result<Vec<SubEntry>, String> {
    let stream = format!("0:s:{}", mpv_track_id.saturating_sub(1));
    let bytes = run_ffmpeg(&["-i", path, "-map", &stream, "-f", "srt", "-"], path).await?;
    let text = String::from_utf8_lossy(&bytes);
    if text.trim().is_empty() { return Err("No subtitle data extracted".into()); }
    Ok(parse_srt_content(&text.replace("\r\n", "\n")))
}

fn parse_ass_content(content: &str) -> Vec<SubEntry> {
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
    entries
}

/// An embedded ASS track: its header and its dialogue, from one ffmpeg run.
///
/// Both come out of the same stream, so asking for them separately demuxed the
/// container twice to read the same bytes — visible in a log as two identical
/// invocations producing identical output back to back. On a small episode that
/// is 7.3 s plus 0.8 s; on a large film it is two full passes.
pub async fn extract_ass_from_video(
    path: &str,
    mpv_track_id: i64,
) -> Result<(String, Vec<SubEntry>), String> {
    let stream = format!("0:s:{}", mpv_track_id.saturating_sub(1));
    let bytes = run_ffmpeg(&["-i", path, "-map", &stream, "-f", "ass", "-"], path).await?;
    let text = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");
    if text.trim().is_empty() {
        return Err("No subtitle data extracted".into());
    }
    Ok((extract_header_from_ass_content(&text)?, parse_ass_content(&text)))
}

/// Extract ASS header from an external .ass file.
pub fn extract_ass_header_from_file(path: &str) -> Result<String, String> {
    extract_header_from_ass_content(&read_text_file(path)?)
}

/// Parse external SRT file.
pub fn extract_from_srt(path: &str) -> Result<Vec<SubEntry>, String> {
    Ok(parse_srt_content(&read_text_file(path)?.replace("\r\n", "\n")))
}

/// Parse an external ASS/SSA file.
pub fn extract_from_ass(path: &str) -> Result<Vec<SubEntry>, String> {
    Ok(parse_ass_content(&read_text_file(path)?.replace("\r\n", "\n")))
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
            '<' => in_html = true, '>' => in_html = false,
            _ if !in_ass && !in_html => out.push(c), _ => {}
        }
    }
    out.replace("\\N", "\n").replace("\\n", "\n")
}

fn parse_srt_content(content: &str) -> Vec<SubEntry> {
    let mut entries = Vec::new();
    for block in content.split("\n\n") {
        let lines: Vec<&str> = block.trim().lines().collect();
        if lines.len() < 3 { continue; }
        let Some((start, end)) = parse_srt_timing(lines[1]) else { continue };
        let text = strip_tags(&lines[2..].join("\n")).trim().to_string();
        if !text.is_empty() { entries.push(SubEntry { start_ms: start, end_ms: end, text, style: "Default".into() }); }
    }
    entries
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
    Some(h * 3_600_000 + m * 60000 + s * 1000 + ms)
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
    h * 3_600_000 + m * 60000 + s * 1000 + cs * 10
}

fn ms_to_ass(ms: u64) -> String {
    format!("{}:{:02}:{:02}.{:02}", ms / 3_600_000, (ms % 3_600_000) / 60000, (ms % 60000) / 1000, (ms % 1000) / 10)
}

fn ms_to_srt(ms: u64) -> String {
    format!("{:02}:{:02}:{:02},{:03}", ms / 3_600_000, (ms % 3_600_000) / 60000, (ms % 60000) / 1000, ms % 1000)
}

fn find_ffmpeg() -> Option<String> {
    // We ship ffmpeg.exe alongside the binary.
    let bundled = "ffmpeg.exe";
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(std::path::Path::to_path_buf)) {
        for sub in ["", "binaries"] {
            let p = dir.join(sub).join(bundled);
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

#[cfg(test)]
mod tests {
    use super::{
        ass_time_to_ms, extract_header_from_ass_content, ms_to_ass, ms_to_srt, parse_ass_content,
        parse_srt_content, srt_time_to_ms, strip_tags,
    };

    #[test]
    fn extraction_timeout_scales_with_the_file_it_reads() {
        use super::timeout_for_size;
        // Small files keep the old flat budget, so a wrong track index still
        // surfaces quickly instead of hanging for minutes.
        assert_eq!(timeout_for_size(0).as_secs(), 30);
        assert_eq!(timeout_for_size(10 * 1024 * 1024).as_secs(), 30);
        // The 10.8 GB file that used to time out: it measures 50s to extract,
        // and the budget has to clear that with room for a slower disk.
        assert!(timeout_for_size(10_859 * 1024 * 1024).as_secs() > 50);
        // Still bounded, so a pathological file errors instead of spinning.
        assert_eq!(timeout_for_size(u64::MAX).as_secs(), 600);
    }

    #[test]
    fn srt_time_parsing() {
        assert_eq!(srt_time_to_ms("00:01:02,500"), Some(62_500));
        assert_eq!(srt_time_to_ms("01:00:00,000"), Some(3_600_000));
        // No fractional part defaults the milliseconds to zero.
        assert_eq!(srt_time_to_ms("00:00:01"), Some(1_000));
        // Short fractions are right-padded: "4" -> 400ms, "05" -> 050ms.
        assert_eq!(srt_time_to_ms("00:00:03,4"), Some(3_400));
        assert_eq!(srt_time_to_ms("00:00:00,05"), Some(50));
        // Malformed input is rejected, not silently zeroed.
        assert_eq!(srt_time_to_ms("bad"), None);
        assert_eq!(srt_time_to_ms("00:01"), None);
    }

    #[test]
    fn ass_time_parsing_uses_centiseconds() {
        assert_eq!(ass_time_to_ms("0:00:01.50"), 1_500);
        // ASS fractions are centiseconds: ".5" is 50cs, not 500ms.
        assert_eq!(ass_time_to_ms("0:00:01.5"), 1_050);
        assert_eq!(ass_time_to_ms("1:02:03.00"), 3_723_000);
        // A malformed ASS time degrades to 0, matching the lenient parser.
        assert_eq!(ass_time_to_ms("nope"), 0);
    }

    #[test]
    fn time_formatting_round_trips() {
        assert_eq!(ms_to_srt(62_500), "00:01:02,500");
        assert_eq!(srt_time_to_ms(&ms_to_srt(62_500)), Some(62_500));
        // ASS carries centisecond precision; round-trip a 10ms-aligned value.
        assert_eq!(ms_to_ass(1_500), "0:00:01.50");
        assert_eq!(ass_time_to_ms(&ms_to_ass(1_500)), 1_500);
    }

    #[test]
    fn strip_tags_removes_ass_and_html_markup() {
        assert_eq!(strip_tags("{\\i1}Hello{\\i0}"), "Hello");
        assert_eq!(strip_tags("<i>Hi</i>"), "Hi");
        assert_eq!(strip_tags("Line1\\NLine2"), "Line1\nLine2");
        // Vector drawing commands rasterize as numbers — dropped entirely.
        assert_eq!(strip_tags("{\\p1}m 0 0 l 1 1{\\p0}"), "");
    }

    #[test]
    fn parses_a_basic_srt_block() {
        let srt = "1\n00:00:01,000 --> 00:00:02,000\nHello world\n";
        let entries = parse_srt_content(srt);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_ms, 1_000);
        assert_eq!(entries[0].end_ms, 2_000);
        assert_eq!(entries[0].text, "Hello world");
    }

    #[test]
    fn parses_a_basic_ass_dialogue() {
        let ass = "Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,{\\i1}Hi{\\i0}";
        let entries = parse_ass_content(ass);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_ms, 1_000);
        assert_eq!(entries[0].style, "Default");
        assert_eq!(entries[0].text, "Hi");
    }

    #[test]
    fn ass_header_stops_at_the_events_format_line() {
        let content = "[Script Info]\n[V4+ Styles]\nFormat: Name\nStyle: Default\n\
[Events]\nFormat: Layer, Start, End, Style, Text\n\
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Hi\n";
        let header = extract_header_from_ass_content(content).unwrap();
        assert!(header.contains("[Events]"));
        assert!(header.contains("Format: Layer"));
        // The header stops before any dialogue line — those are rewritten.
        assert!(!header.contains("Dialogue"));
    }
}
