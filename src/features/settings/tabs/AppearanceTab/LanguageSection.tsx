import { useState } from "react";
import { LANGUAGE_NAMES, LanguageSetting, useI18n } from "../../../../i18n/I18nProvider";

interface LanguageSectionProps {
  value: LanguageSetting;
  disabled: boolean;
  onChange: (language: LanguageSetting) => Promise<boolean>;
}

export function LanguageSection({ value, disabled, onChange }: LanguageSectionProps) {
  const { m, language } = useI18n();
  const [saving, setSaving] = useState(false);
  const t = m.settings.appearance;

  // "system" shows the language Windows resolves to right now.
  const options: { value: LanguageSetting; label: string }[] = [
    { value: "system", label: value === "system" ? t.system(LANGUAGE_NAMES[language]) : t.systemPlain },
    { value: "en", label: LANGUAGE_NAMES.en },
    { value: "pt-BR", label: LANGUAGE_NAMES["pt-BR"] },
  ];

  const handleChange = async (next: LanguageSetting) => {
    setSaving(true);
    await onChange(next);
    setSaving(false);
  };

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">{t.languageTitle}</h3>
          <p className="card-description">{t.languageDescription}</p>
        </div>
      </div>

      <div className="backup-body">
        <select
          className="settings-select language-select"
          aria-label={t.languageTitle}
          value={value}
          disabled={disabled || saving}
          onChange={(e) => handleChange(e.target.value as LanguageSetting)}
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>
    </section>
  );
}
