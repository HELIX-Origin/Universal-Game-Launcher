/**
 * Registry of every UI locale. The list is the union of the UI languages
 * offered by the platforms the launcher supports (Steam, Epic, GOG, itch.io,
 * Ubisoft Connect, EA, Origin, Xbox / Microsoft Store, Amazon, Battle.net,
 * GeForce NOW, xbox.com). See docs/research/localization.md for sources.
 *
 * Tag convention (BCP-47): bare language unless variants differ; Chinese uses
 * script subtags (zh-Hans / zh-Hant), Norwegian is `nb`, Latin-American
 * Spanish is `es-419`, Serbian is split by script.
 */

export type TextDirection = "ltr" | "rtl";

export interface LocaleInfo {
  /** BCP-47 tag; also the dictionary file name in ./locales. */
  code: string;
  /** English name (for search / docs). */
  englishName: string;
  /** Endonym shown in the language picker. */
  nativeName: string;
  dir: TextDirection;
  /**
   * Locale whose dictionary is used for keys this one does not define.
   * Regional overlays (en-GB, fr-CA, es-419, pt-PT) only override what differs.
   * Defaults to "en".
   */
  fallback?: string;
}

export const DEFAULT_LOCALE = "en";

export const LOCALES: readonly LocaleInfo[] = [
  { code: "ar", englishName: "Arabic", nativeName: "العربية", dir: "rtl" },
  { code: "bg", englishName: "Bulgarian", nativeName: "български", dir: "ltr" },
  { code: "bs", englishName: "Bosnian", nativeName: "bosanski", dir: "ltr", fallback: "hr" },
  { code: "ca", englishName: "Catalan", nativeName: "català", dir: "ltr", fallback: "es" },
  { code: "cs", englishName: "Czech", nativeName: "čeština", dir: "ltr" },
  { code: "cy", englishName: "Welsh", nativeName: "Cymraeg", dir: "ltr" },
  { code: "da", englishName: "Danish", nativeName: "dansk", dir: "ltr" },
  { code: "de", englishName: "German", nativeName: "Deutsch", dir: "ltr" },
  { code: "el", englishName: "Greek", nativeName: "Ελληνικά", dir: "ltr" },
  { code: "en", englishName: "English (US)", nativeName: "English (US)", dir: "ltr" },
  { code: "en-GB", englishName: "English (UK)", nativeName: "English (UK)", dir: "ltr", fallback: "en" },
  { code: "eo", englishName: "Esperanto", nativeName: "Esperanto", dir: "ltr" },
  { code: "es", englishName: "Spanish (Spain)", nativeName: "español (España)", dir: "ltr" },
  { code: "es-419", englishName: "Spanish (Latin America)", nativeName: "español (Latinoamérica)", dir: "ltr", fallback: "es" },
  { code: "et", englishName: "Estonian", nativeName: "eesti", dir: "ltr" },
  { code: "fa", englishName: "Persian", nativeName: "فارسی", dir: "rtl" },
  { code: "fi", englishName: "Finnish", nativeName: "suomi", dir: "ltr" },
  { code: "fr", englishName: "French", nativeName: "français", dir: "ltr" },
  { code: "fr-CA", englishName: "French (Canada)", nativeName: "français (Canada)", dir: "ltr", fallback: "fr" },
  { code: "ga", englishName: "Irish", nativeName: "Gaeilge", dir: "ltr" },
  { code: "he", englishName: "Hebrew", nativeName: "עברית", dir: "rtl" },
  { code: "hi", englishName: "Hindi", nativeName: "हिन्दी", dir: "ltr" },
  { code: "hr", englishName: "Croatian", nativeName: "hrvatski", dir: "ltr" },
  { code: "hu", englishName: "Hungarian", nativeName: "magyar", dir: "ltr" },
  { code: "id", englishName: "Indonesian", nativeName: "Bahasa Indonesia", dir: "ltr" },
  { code: "is", englishName: "Icelandic", nativeName: "íslenska", dir: "ltr" },
  { code: "it", englishName: "Italian", nativeName: "italiano", dir: "ltr" },
  { code: "ja", englishName: "Japanese", nativeName: "日本語", dir: "ltr" },
  { code: "ka", englishName: "Georgian", nativeName: "ქართული", dir: "ltr" },
  { code: "ko", englishName: "Korean", nativeName: "한국어", dir: "ltr" },
  { code: "lt", englishName: "Lithuanian", nativeName: "lietuvių", dir: "ltr" },
  { code: "lv", englishName: "Latvian", nativeName: "latviešu", dir: "ltr" },
  { code: "mk", englishName: "Macedonian", nativeName: "македонски", dir: "ltr" },
  { code: "nb", englishName: "Norwegian Bokmål", nativeName: "norsk bokmål", dir: "ltr" },
  { code: "nl", englishName: "Dutch", nativeName: "Nederlands", dir: "ltr" },
  { code: "pl", englishName: "Polish", nativeName: "polski", dir: "ltr" },
  { code: "pt-BR", englishName: "Portuguese (Brazil)", nativeName: "português (Brasil)", dir: "ltr" },
  { code: "pt-PT", englishName: "Portuguese (Portugal)", nativeName: "português (Portugal)", dir: "ltr", fallback: "pt-BR" },
  { code: "ro", englishName: "Romanian", nativeName: "română", dir: "ltr" },
  { code: "ru", englishName: "Russian", nativeName: "русский", dir: "ltr" },
  { code: "sk", englishName: "Slovak", nativeName: "slovenčina", dir: "ltr", fallback: "cs" },
  { code: "sl", englishName: "Slovenian", nativeName: "slovenščina", dir: "ltr" },
  { code: "sq", englishName: "Albanian", nativeName: "shqip", dir: "ltr" },
  { code: "sr-Cyrl", englishName: "Serbian (Cyrillic)", nativeName: "српски", dir: "ltr" },
  { code: "sr-Latn", englishName: "Serbian (Latin)", nativeName: "srpski", dir: "ltr", fallback: "sr-Cyrl" },
  { code: "sv", englishName: "Swedish", nativeName: "svenska", dir: "ltr" },
  { code: "th", englishName: "Thai", nativeName: "ไทย", dir: "ltr" },
  { code: "tr", englishName: "Turkish", nativeName: "Türkçe", dir: "ltr" },
  { code: "uk", englishName: "Ukrainian", nativeName: "українська", dir: "ltr" },
  { code: "vi", englishName: "Vietnamese", nativeName: "Tiếng Việt", dir: "ltr" },
  { code: "zh-Hans", englishName: "Chinese (Simplified)", nativeName: "简体中文", dir: "ltr" },
  { code: "zh-Hant", englishName: "Chinese (Traditional)", nativeName: "繁體中文", dir: "ltr" },
];

/**
 * Locales whose dictionary is a partial overlay on their `fallback`
 * (only the strings that differ regionally). All others must be complete.
 */
export const OVERLAY_LOCALES = new Set(["en-GB", "fr-CA", "es-419", "pt-PT"]);

/** Locales whose dictionary is generated at runtime from another one. */
export const DERIVED_LOCALES = new Set(["sr-Latn"]);

const byLower = new Map(LOCALES.map((l) => [l.code.toLowerCase(), l]));

export function getLocaleInfo(code: string): LocaleInfo | undefined {
  return byLower.get(code.toLowerCase());
}

/** Aliases for tags reported by operating systems / browsers / store apps. */
const ALIASES: Record<string, string> = {
  zh: "zh-Hans",
  "zh-cn": "zh-Hans",
  "zh-sg": "zh-Hans",
  "zh-my": "zh-Hans",
  "zh-tw": "zh-Hant",
  "zh-hk": "zh-Hant",
  "zh-mo": "zh-Hant",
  no: "nb",
  nn: "nb",
  pt: "pt-PT",
  sr: "sr-Cyrl",
  iw: "he",
  in: "id",
  ji: "yi",
  fil: "en",
  // Steam API language names (e.g. from `steam://` settings or Proton envs).
  schinese: "zh-Hans",
  tchinese: "zh-Hant",
  koreana: "ko",
  brazilian: "pt-BR",
  latam: "es-419",
  vn: "vi",
};

const ENGLISH_GB_REGIONS = new Set(["gb", "uk", "ie", "au", "nz", "in", "za", "sg", "hk", "mt"]);

/**
 * Map an arbitrary BCP-47 tag to a supported locale code, or `undefined`.
 * Handles case, `_` separators, script/region variants and aliases.
 */
export function matchLocale(tag: string | null | undefined): string | undefined {
  if (!tag) return undefined;
  const norm = tag.trim().replace(/_/g, "-").toLowerCase();
  if (!norm) return undefined;
  const exact = byLower.get(norm);
  if (exact) return exact.code;
  const alias = ALIASES[norm];
  if (alias && byLower.has(alias.toLowerCase())) return byLower.get(alias.toLowerCase())!.code;

  const parts = norm.split("-");
  const lang = parts[0];
  const rest = parts.slice(1);
  const script = rest.find((p) => p.length === 4);
  const region = rest.find((p) => p.length === 2 || /^\d{3}$/.test(p));

  if (lang === "zh") {
    if (script === "hant" || (region && ["tw", "hk", "mo"].includes(region))) return "zh-Hant";
    return "zh-Hans";
  }
  if (lang === "sr") return script === "latn" || region === "me" ? "sr-Latn" : "sr-Cyrl";
  if (lang === "en") return region && ENGLISH_GB_REGIONS.has(region) ? "en-GB" : "en";
  if (lang === "fr" && region === "ca") return "fr-CA";
  if (lang === "pt") return region === "br" ? "pt-BR" : "pt-PT";
  if (lang === "es") return region && region !== "es" && region !== "gq" ? "es-419" : "es";

  const base = byLower.get(lang) ?? (ALIASES[lang] ? byLower.get(ALIASES[lang].toLowerCase()) : undefined);
  return base?.code;
}

/** First supported locale among the user's preferred tags, else English. */
export function negotiateLocale(preferred: readonly string[]): string {
  for (const tag of preferred) {
    const m = matchLocale(tag);
    if (m) return m;
  }
  return DEFAULT_LOCALE;
}

/** Lookup chain for a locale: itself, its fallbacks, then English. */
export function fallbackChain(code: string): string[] {
  const chain: string[] = [];
  let current: string | undefined = getLocaleInfo(code)?.code ?? DEFAULT_LOCALE;
  while (current && !chain.includes(current)) {
    chain.push(current);
    current = getLocaleInfo(current)?.fallback ?? (current === DEFAULT_LOCALE ? undefined : DEFAULT_LOCALE);
  }
  return chain;
}
