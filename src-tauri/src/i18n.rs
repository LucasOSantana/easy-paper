//! Interface language for text produced by the backend: tray menu, dialog
//! titles and error messages shown in the UI. The React UI has its own
//! message files (`src/i18n`) and asks the backend which language to use
//! (`get_language`), so both sides always agree.
//!
//! Text is written next to its use with [`tr!`], one argument per language.
//! Adding a language means adding a [`Language`] variant and an arm to the
//! macro; the compiler then points at every call that needs a translation.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::Serialize;

use crate::models::AppConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Language {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "pt-BR")]
    PtBr,
}

/// Values accepted in `config.language`.
pub const LANGUAGE_SYSTEM: &str = "system";
pub const SUPPORTED_SETTINGS: [&str; 3] = [LANGUAGE_SYSTEM, "en", "pt-BR"];

static CURRENT: AtomicU8 = AtomicU8::new(0);

impl Language {
    fn to_u8(self) -> u8 {
        match self {
            Language::En => 0,
            Language::PtBr => 1,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Language::PtBr,
            _ => Language::En,
        }
    }
}

/// The language currently used by the backend.
pub fn current() -> Language {
    Language::from_u8(CURRENT.load(Ordering::Relaxed))
}

/// Resolves `config.language` ("system", "en" or "pt-BR") and makes it the
/// current language. Returns it.
pub fn apply_config(config: &AppConfig) -> Language {
    let language = resolve(&config.language);
    CURRENT.store(language.to_u8(), Ordering::Relaxed);
    language
}

pub fn resolve(setting: &str) -> Language {
    match setting {
        "en" => Language::En,
        "pt-BR" => Language::PtBr,
        _ => system_language(),
    }
}

/// Windows display language: Portuguese (any region) gets pt-BR, anything
/// else English.
#[cfg(windows)]
pub fn system_language() -> Language {
    const LANG_PORTUGUESE: u16 = 0x16;
    let lang_id = unsafe { windows::Win32::Globalization::GetUserDefaultUILanguage() };
    // The primary language is in the low 10 bits of a LANGID.
    if lang_id & 0x3ff == LANG_PORTUGUESE {
        Language::PtBr
    } else {
        Language::En
    }
}

#[cfg(not(windows))]
pub fn system_language() -> Language {
    Language::En
}

/// Picks the text for the current language. Each argument is only evaluated
/// when it is the one used, so `format!` calls cost nothing otherwise.
///
/// ```ignore
/// tr!(en: "Could not open Explorer.", pt: "Não foi possível abrir o Explorer.")
/// tr!(en: format!("Could not read {}", path), pt: format!("Não foi possível ler {}", path))
/// ```
#[macro_export]
macro_rules! tr {
    (en: $en:expr, pt: $pt:expr $(,)?) => {
        match $crate::i18n::current() {
            $crate::i18n::Language::En => ::std::string::String::from($en),
            $crate::i18n::Language::PtBr => ::std::string::String::from($pt),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_settings_resolve_without_the_system() {
        assert_eq!(resolve("en"), Language::En);
        assert_eq!(resolve("pt-BR"), Language::PtBr);
    }

    #[test]
    fn macro_follows_the_current_language() {
        let mut config = AppConfig::default();
        config.language = "pt-BR".into();
        apply_config(&config);
        assert_eq!(tr!(en: "Hello", pt: "Olá"), "Olá");

        config.language = "en".into();
        apply_config(&config);
        assert_eq!(tr!(en: format!("{} files", 2), pt: format!("{} arquivos", 2)), "2 files");
    }
}
