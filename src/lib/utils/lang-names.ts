const names: Record<string, string> = {
  eng: "English", por: "Portuguese", spa: "Spanish", fre: "French", fra: "French",
  deu: "German", ger: "German", ita: "Italian", jpn: "Japanese", kor: "Korean",
  zho: "Chinese", chi: "Chinese", rus: "Russian", ara: "Arabic", hin: "Hindi",
  tur: "Turkish", pol: "Polish", nld: "Dutch", dut: "Dutch", swe: "Swedish",
  nor: "Norwegian", dan: "Danish", fin: "Finnish", ces: "Czech", cze: "Czech",
  slk: "Slovak", slo: "Slovak", hun: "Hungarian", ron: "Romanian", rum: "Romanian",
  bul: "Bulgarian", hrv: "Croatian", srp: "Serbian", ukr: "Ukrainian",
  ell: "Greek", gre: "Greek", heb: "Hebrew", tha: "Thai", vie: "Vietnamese",
  ind: "Indonesian", msa: "Malay", may: "Malay", fil: "Filipino", cat: "Catalan",
  eus: "Basque", baq: "Basque", glg: "Galician", lat: "Latin",
  pt: "Portuguese", en: "English", es: "Spanish", fr: "French", de: "German",
  it: "Italian", ja: "Japanese", ko: "Korean", zh: "Chinese", ru: "Russian",
  ar: "Arabic", hi: "Hindi", tr: "Turkish", pl: "Polish", nl: "Dutch",
  sv: "Swedish", no: "Norwegian", da: "Danish", fi: "Finnish",
  pob: "Portuguese (BR)", pb: "Portuguese (BR)",
};

export function langName(code: string): string {
  if (!code) return "";
  return names[code.toLowerCase()] ?? code;
}
