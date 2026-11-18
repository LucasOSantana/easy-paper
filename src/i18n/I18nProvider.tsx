import { createContext, ReactNode, useCallback, useContext, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../services/tauriCommands";
import { CONFIG_CHANGED_EVENT } from "../types/sync";
import { en, Messages } from "./locales/en";
import { ptBR } from "./locales/ptBR";

/** Languages the interface is translated to. */
export type Language = "en" | "pt-BR";
/** What the user picks in Settings: a language, or follow Windows. */
export type LanguageSetting = "system" | Language;

export const MESSAGES: Record<Language, Messages> = {
  en,
  "pt-BR": ptBR,
};

/** Each language's own name, shown in the language picker. */
export const LANGUAGE_NAMES: Record<Language, string> = {
  en: "English",
  "pt-BR": "Português (Brasil)",
};

interface I18nValue {
  /** Language in use (the "system" setting already resolved by the backend). */
  language: Language;
  /** Messages in the current language. */
  m: Messages;
  /** Date and time in the current language's format. */
  formatDateTime: (ms: number) => string;
  /** "just now", "5 min ago", "2 h ago" or "on <date>". */
  formatTimeAgo: (ms: number, now?: number) => string;
}

const I18nContext = createContext<I18nValue | null>(null);

/** Best guess before the backend answers, so the first paint is rarely in the wrong language. */
function guessLanguage(): Language {
  return navigator.language.toLowerCase().startsWith("pt") ? "pt-BR" : "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguage] = useState<Language>(guessLanguage);

  const refresh = useCallback(() => {
    TauriService.getLanguage()
      .then((resolved) => setLanguage(resolved))
      .catch(console.error);
  }, []);

  useEffect(() => {
    refresh();
    // The language may change from Settings, a restored backup or sync.
    const unlisten = listen(CONFIG_CHANGED_EVENT, refresh);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, [refresh]);

  useEffect(() => {
    document.documentElement.lang = language;
  }, [language]);

  const value = useMemo<I18nValue>(() => {
    const m = MESSAGES[language];
    const dateFormat = new Intl.DateTimeFormat(language, { dateStyle: "short", timeStyle: "short" });
    const formatDateTime = (ms: number) => dateFormat.format(new Date(ms));

    const formatTimeAgo = (ms: number, now = Date.now()) => {
      const seconds = Math.max(0, Math.round((now - ms) / 1000));
      if (seconds < 45) return m.common.justNow;
      const minutes = Math.round(seconds / 60);
      if (minutes < 60) return m.common.minutesAgo(minutes);
      const hours = Math.round(minutes / 60);
      if (hours < 24) return m.common.hoursAgo(hours);
      return m.common.onDate(formatDateTime(ms));
    };

    return { language, m, formatDateTime, formatTimeAgo };
  }, [language]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const value = useContext(I18nContext);
  if (!value) throw new Error("useI18n must be used inside <I18nProvider>");
  return value;
}
