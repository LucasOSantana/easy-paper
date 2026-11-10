import { openUrl } from "@tauri-apps/plugin-opener";
import { useUpdateStatus } from "../../../hooks/useUpdateStatus";
import { UpdateProgress } from "../../../types/update";

interface UpdateFooterProps {
  onFeedback: (type: "success" | "error", message: string) => void;
}

function describeProgress(progress: UpdateProgress | null): { label: string; percent: number | null } {
  if (!progress) return { label: "Preparing…", percent: null };
  const mb = (bytes: number) => (bytes / 1024 / 1024).toFixed(1);
  if (!progress.total) return { label: `${mb(progress.downloaded)} MB`, percent: null };
  const percent = Math.min(100, Math.round((progress.downloaded / progress.total) * 100));
  return { label: percent >= 100 ? "Installing…" : `Downloading ${percent}%`, percent };
}

/** Sidebar footer: current version, manual check and, when there is one, the new release. */
export function UpdateFooter({ onFeedback }: UpdateFooterProps) {
  const { currentVersion, info, checking, checkNow, installing, progress, installNow } =
    useUpdateStatus(onFeedback);
  const status = describeProgress(progress);

  return (
    <div className="sidebar-footer">
      {info?.update_available && (
        <div className="update-card" role="status">
          <span className="update-eyebrow">Update available</span>
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
                Install &amp; restart
              </button>
              <button className="link-button" type="button" onClick={() => openUrl(info.release_url)}>
                What's new
              </button>
            </div>
          )}
        </div>
      )}

      <div className="sidebar-version-row">
        <span className="sidebar-version">{currentVersion ? `v${currentVersion}` : ""}</span>
        <button className="link-button" type="button" disabled={checking || installing} onClick={checkNow}>
          {checking ? "Checking…" : "Check for updates"}
        </button>
      </div>
    </div>
  );
}
