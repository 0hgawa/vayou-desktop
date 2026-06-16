import { langName } from "./lang-names";

// Short, friendly format tag for a track codec ("subrip" -> "SRT",
// "ac3" -> "AC3"). Empty for codecs we don't tag. Covers both subtitle and audio
// codecs so every track row can show its format like PotPlayer does.
export function trackFormat(codec: string): string {
  switch (codec.toLowerCase()) {
    // subtitles
    case "subrip":
    case "srt": return "SRT";
    case "ass": return "ASS";
    case "ssa": return "SSA";
    case "webvtt": return "VTT";
    case "hdmv_pgs_subtitle":
    case "pgs": return "PGS";
    case "dvd_subtitle": return "VobSub";
    case "dvb_subtitle": return "DVB";
    case "mov_text": return "TX3G";
    case "microdvd": return "SUB";
    // audio
    case "aac": return "AAC";
    case "ac3": return "AC3";
    case "eac3": return "E-AC3";
    case "dts": return "DTS";
    case "truehd": return "TrueHD";
    case "flac": return "FLAC";
    case "opus": return "Opus";
    case "mp3": return "MP3";
    case "vorbis": return "Vorbis";
    case "pcm_s16le":
    case "pcm_s24le":
    case "pcm_s32le": return "PCM";
    default: return "";
  }
}

// Friendly track label: "German", "English · SDH", "Korean (SRT)" — the full
// language name (mapped from the code), the title when it adds information, and
// the format tag from the codec. Falls back to the title, then "Track N".
// Mirrors the Slint build's `track_label`.
export function trackLabel(title: string, lang: string, codec: string, id: number): string {
  const name = langName(lang);
  let base: string;
  if (name && title && title.toLowerCase() !== name.toLowerCase()) base = `${name} · ${title}`;
  else if (name) base = name;
  else if (title) base = title;
  else base = `Track ${id}`;
  const fmt = trackFormat(codec);
  return fmt ? `${base} (${fmt})` : base;
}
