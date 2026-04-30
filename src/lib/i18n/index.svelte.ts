import en from "./en";
import pt from "./pt";
import es from "./es";
import fr from "./fr";
import de from "./de";
import it from "./it";
import ja from "./ja";
import ko from "./ko";
import zh from "./zh";
import ru from "./ru";
import hi from "./hi";
import ar from "./ar";

export type Dict = Record<keyof typeof en, string>;

const dicts: Record<string, Dict> = { en, pt, es, fr, de, it, ja, ko, zh, ru, hi, ar };

let locale = $state("en");
let dict = $derived(dicts[locale] ?? dicts.en);

export function t(): Dict {
  return dict;
}

export function setLocale(code: string) {
  if (dicts[code]) locale = code;
}

export function getLocale(): string {
  return locale;
}
