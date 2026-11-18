import { BackupSection } from "./BackupSection";
import { SyncSection } from "./SyncSection";
import { useAppSettings } from "../../../../hooks/useAppSettings";
import { useI18n } from "../../../../i18n/I18nProvider";

interface BackupTabProps {
  settings: ReturnType<typeof useAppSettings>;
}

export function BackupTab({ settings }: BackupTabProps) {
  const { config, showFeedback } = settings;
  const { m } = useI18n();

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">{m.settings.frame(4)}</span>
        <h2 className="content-header-title">{m.settings.backup.title}</h2>
        <p className="content-header-subtitle">{m.settings.backup.subtitle}</p>
      </div>

      <BackupSection onFeedback={showFeedback} />
      <SyncSection folder={config.sync.folder} onFeedback={showFeedback} />
    </div>
  );
}
