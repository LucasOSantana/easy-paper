import { TransparencySection } from "./TransparencySection";
import { useAppSettings } from "../../../../hooks/useAppSettings";
import { useI18n } from "../../../../i18n/I18nProvider";

interface TaskbarTabProps {
  settings: ReturnType<typeof useAppSettings>;
}

export function TaskbarTab({ settings }: TaskbarTabProps) {
  const { config, loading, taskbarUpdating, changeTaskbarTransparent } = settings;
  const { m } = useI18n();

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">{m.settings.frame(2)}</span>
        <h2 className="content-header-title">{m.settings.taskbar.title}</h2>
        <p className="content-header-subtitle">{m.settings.taskbar.subtitle}</p>
      </div>

      <TransparencySection
        transparent={config.taskbar_transparent}
        disabled={loading || taskbarUpdating}
        onChange={changeTaskbarTransparent}
      />
    </div>
  );
}
