import { openUrl } from "@tauri-apps/plugin-opener";
import { useUpdateStatus } from "../../../hooks/useUpdateStatus";

interface UpdateFooterProps {
  onFeedback: (type: "success" | "error", message: string) => void;
}

/** Sidebar footer: current version, manual check and, when there is one, the new release. */
export function UpdateFooter({ onFeedback }: UpdateFooterProps) {
  const { currentVersion, info, checking, checkNow } = useUpdateStatus(onFeedback);

  return (
    <div className="sidebar-footer">
      {info?.update_available && (
        <div className="update-card" role="status">
          <span className="update-eyebrow">Update available</span>
          <span className="update-version">v{info.latest_version}</span>
          <div className="update-actions">
            <button
              className="btn-primary update-download"
              type="button"
              onClick={() => openUrl(info.download_url ?? info.release_url)}
            >
              Download
            </button>
            <button className="link-button" type="button" onClick={() => openUrl(info.release_url)}>
              What's new
            </button>
          </div>
        </div>
      )}

      <div className="sidebar-version-row">
        <span className="sidebar-version">{currentVersion ? `v${currentVersion}` : ""}</span>
        <button className="link-button" type="button" disabled={checking} onClick={checkNow}>
          {checking ? "Checking…" : "Check for updates"}
        </button>
      </div>
    </div>
  );
}
