import { openUrl } from "@tauri-apps/plugin-opener";
import { useUpdateStatus } from "../../../hooks/useUpdateStatus";
import { UpdateProgress } from "../../../types/update";
import { Messages } from "../../../i18n/locales/en";
import { useI18n } from "../../../i18n/I18nProvider";

interface UpdateFooterProps {
  onFeedback: (type: "success" | "error", message: string) => void;
}

function describeProgress(
  progress: UpdateProgress | null,
  t: Messages["settings"]["update"]
): { label: string; percent: number | null } {
  if (!progress) return { label: t.preparing, percent: null };
  const mb = (bytes: number) => (bytes / 1024 / 1024).toFixed(1);
  if (!progress.total) return { label: `${mb(progress.downloaded)} MB`, percent: null };
  const percent = Math.min(100, Math.round((progress.downloaded / progress.total) * 100));
  return { label: percent >= 100 ? t.installing : t.downloading(percent), percent };
}

/** Sidebar footer: current version, manual check and, when there is one, the new release. */
export function UpdateFooter({ onFeedback }: UpdateFooterProps) {
  const { currentVersion, info, checking, checkNow, installing, progress, installNow } =
    useUpdateStatus(onFeedback);
  const { m } = useI18n();
  const t = m.settings.update;
  const status = describeProgress(progress, t);

  return (
    <div className="sidebar-footer">
      {info?.update_available && (
        <div className="update-card" role="status">
          <span className="update-eyebrow">{t.available}</span>
          <span className="update-version">v{info.latest_version}</span>

          {installing ? (
            <>
              <div className="update-progress" aria-hidden="true">
                <div className="update-progress-bar" style={{ width: `${status.percent ?? 100}%` }} />
              </div>
              <span className="update-progress-label">{status.label}</span>
            </>
          ) : (
            <div className="update-actions">
              <button className="btn-primary update-download" type="button" onClick={installNow}>
                {t.install}
              </button>
              <button className="link-button" type="button" onClick={() => openUrl(info.release_url)}>
                {t.whatsNew}
              </button>
            </div>
          )}
        </div>
      )}

      <div className="sidebar-version-row">
        <span className="sidebar-version">{currentVersion ? `v${currentVersion}` : ""}</span>
        <button className="link-button" type="button" disabled={checking || installing} onClick={checkNow}>
          {checking ? t.checking : t.check}
        </button>
      </div>
    </div>
  );
}
