import { useAppSettings } from "../../../../hooks/useAppSettings";
import { useI18n } from "../../../../i18n/I18nProvider";
import { LanguageSection } from "./LanguageSection";

interface AppearanceTabProps {
  settings: ReturnType<typeof useAppSettings>;
}

export function AppearanceTab({ settings }: AppearanceTabProps) {
  const { config, loading, changeLanguage } = settings;
  const { m } = useI18n();

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">{m.settings.frame(5)}</span>
        <h2 className="content-header-title">{m.settings.appearance.title}</h2>
        <p className="content-header-subtitle">{m.settings.appearance.subtitle}</p>
      </div>

      <LanguageSection value={config.language} disabled={loading} onChange={changeLanguage} />
    </div>
  );
}
