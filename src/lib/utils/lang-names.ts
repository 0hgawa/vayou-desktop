// Base ISO-639-1/2 codes whose region doesn't change the displayed name.
const names: Record<string, string> = {
  eng: "English", en: "English",
  por: "Portuguese", pt: "Portuguese",
  spa: "Spanish", es: "Spanish",
  fre: "French", fra: "French", fr: "French",
  deu: "German", ger: "German", de: "German",
  ita: "Italian", it: "Italian",
  jpn: "Japanese", ja: "Japanese",
  kor: "Korean", ko: "Korean",
  zho: "Chinese", chi: "Chinese", zh: "Chinese",
  rus: "Russian", ru: "Russian",
  ara: "Arabic", ar: "Arabic",
  hin: "Hindi", hi: "Hindi",
  tur: "Turkish", tr: "Turkish",
  pol: "Polish", pl: "Polish",
  nld: "Dutch", dut: "Dutch", nl: "Dutch",
  swe: "Swedish", sv: "Swedish",
  nor: "Norwegian", nob: "Norwegian", no: "Norwegian", nb: "Norwegian",
  dan: "Danish", da: "Danish",
  fin: "Finnish", fi: "Finnish",
  ces: "Czech", cze: "Czech", cs: "Czech",
  slk: "Slovak", slo: "Slovak", sk: "Slovak",
  slv: "Slovenian", sl: "Slovenian",
  hun: "Hungarian", hu: "Hungarian",
  ron: "Romanian", rum: "Romanian", ro: "Romanian",
  bul: "Bulgarian", bg: "Bulgarian",
  hrv: "Croatian", hr: "Croatian",
  srp: "Serbian", scc: "Serbian", sr: "Serbian",
  ukr: "Ukrainian", uk: "Ukrainian",
  ell: "Greek", gre: "Greek", el: "Greek",
  heb: "Hebrew", he: "Hebrew", iw: "Hebrew",
  tha: "Thai", th: "Thai",
  vie: "Vietnamese", vi: "Vietnamese",
  ind: "Indonesian", id: "Indonesian",
  msa: "Malay", may: "Malay", ms: "Malay",
  fil: "Filipino", tl: "Filipino",
  cat: "Catalan", ca: "Catalan",
  eus: "Basque", baq: "Basque", eu: "Basque",
  glg: "Galician", gl: "Galician",
  lit: "Lithuanian", lt: "Lithuanian",
  lav: "Latvian", lv: "Latvian",
  est: "Estonian", et: "Estonian",
  isl: "Icelandic", ice: "Icelandic", is: "Icelandic",
  kat: "Georgian", geo: "Georgian", ka: "Georgian",
  hye: "Armenian", arm: "Armenian", hy: "Armenian",
  aze: "Azerbaijani", az: "Azerbaijani",
  kaz: "Kazakh", kk: "Kazakh",
  fas: "Persian", per: "Persian", fa: "Persian",
  urd: "Urdu", ur: "Urdu",
  ben: "Bengali", bn: "Bengali",
  tam: "Tamil", ta: "Tamil",
  tel: "Telugu", te: "Telugu",
  lat: "Latin", la: "Latin",
};

// Regional variants whose region changes the displayed name.
const regional: Record<string, string> = {
  "pt-br": "Portuguese (BR)", pob: "Portuguese (BR)", pb: "Portuguese (BR)",
  "pt-pt": "Portuguese (PT)",
  "es-419": "Spanish (LA)", "es-la": "Spanish (LA)", "es-mx": "Spanish (LA)",
  "es-ar": "Spanish (LA)", "es-co": "Spanish (LA)",
  "zh-cn": "Chinese (Simplified)", "zh-hans": "Chinese (Simplified)", "zh-sg": "Chinese (Simplified)",
  "zh-tw": "Chinese (Traditional)", "zh-hk": "Chinese (Traditional)", "zh-hant": "Chinese (Traditional)",
};

// Full language name for an ISO-639 code, ignoring region (en-US -> English)
// except where the region changes the name (pt-BR, zh-TW). Falls back to the raw
// code. Mirrors the Slint build's `lang_name`.
export function langName(code: string): string {
  if (!code) return "";
  const c = code.toLowerCase().replace(/_/g, "-");
  if (regional[c]) return regional[c];
  return names[c.split("-")[0]] ?? code;
}
