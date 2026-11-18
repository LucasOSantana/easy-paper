import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../../../../services/tauriCommands";
import { SYNC_STATUS_EVENT, SyncFolderInfo, SyncStatus } from "../../../../types/sync";
import { Messages } from "../../../../i18n/locales/en";
import { useI18n } from "../../../../i18n/I18nProvider";

interface SyncSectionProps {
  folder: string;
  onFeedback: (type: "success" | "error", message: string) => void;
}

/** A folder picked that already holds settings, waiting for the user's choice. */
type PendingFolder = { folder: string; info: SyncFolderInfo };

function describeStatus(
  status: SyncStatus | null,
  t: Messages["settings"]["sync"],
  timeAgo: (ms: number) => string
): string {
  if (!status || status.last_checked_at === null) return t.waiting;
  const when = timeAgo(status.last_checked_at);
  switch (status.last_action) {
    case "pulled":
      return t.pulled(status.remote_device, when);
    case "pushed":
      return t.pushed(when);
    default:
      return t.upToDate(when);
  }
}

export function SyncSection({ folder, onFeedback }: SyncSectionProps) {
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [pending, setPending] = useState<PendingFolder | null>(null);
  const [busy, setBusy] = useState(false);
  const [now, setNow] = useState(() => Date.now());
  const connected = folder.trim() !== "";
  const { m, formatDateTime, formatTimeAgo } = useI18n();
  const t = m.settings.sync;

  useEffect(() => {
    const refresh = () => TauriService.getSyncStatus().then(setStatus).catch(console.error);
    refresh();
    // Changes are pushed as events; the check time alone is not, so read it
    // while this tab is open.
    const unlisten = listen<SyncStatus>(SYNC_STATUS_EVENT, (event) => setStatus(event.payload));
    const timer = setInterval(() => {
      setNow(Date.now());
      refresh();
    }, 30_000);
    return () => {
      unlisten.then((stop) => stop());
      clearInterval(timer);
    };
  }, []);

  const run = async (task: () => Promise<void>, fallbackError: string) => {
    try {
      setBusy(true);
      await task();
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : fallbackError);
    } finally {
      setBusy(false);
      setNow(Date.now());
    }
  };

  const connect = (target: string, useFolderSettings: boolean) =>
    run(async () => {
      setStatus(await TauriService.connectSyncFolder(target, useFolderSettings));
      setPending(null);
      onFeedback("success", t.on);
    }, t.turnOnFailed);

  const handlePickFolder = () =>
    run(async () => {
      const picked = await TauriService.pickSyncFolder();
      if (!picked) return;
      const info = await TauriService.inspectSyncFolder(picked);
      if (info) {
        setPending({ folder: picked, info });
      } else {
        setStatus(await TauriService.connectSyncFolder(picked, false));
        onFeedback("success", t.on);
      }
    }, t.folderFailed);

  const handleSyncNow = () =>
    run(async () => {
      setStatus(await TauriService.syncNow());
    }, t.syncFailed);

  const handleDisconnect = () =>
    run(async () => {
      await TauriService.disconnectSync();
      setStatus(null);
      onFeedback("success", t.off);
    }, t.turnOffFailed);

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 12a9 9 0 0 1-15.5 6.2" />
            <path d="M3 12a9 9 0 0 1 15.5-6.2" />
            <polyline points="18.5 2 18.5 6 14.5 6" />
            <polyline points="5.5 22 5.5 18 9.5 18" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">{t.title}</h3>
          <p className="card-description">{t.description}</p>
        </div>
      </div>

      {pending ? (
        <div className="sync-choice" role="group" aria-label={t.choiceLabel}>
          <p className="sync-choice-text">
            {t.existing(pending.info.device, formatDateTime(pending.info.updated_at), pending.info.favorites)}
          </p>
          <div className="backup-actions">
            <button className="btn-primary" type="button" disabled={busy} onClick={() => connect(pending.folder, true)}>
              {t.useFolder}
            </button>
            <button className="btn-secondary" type="button" disabled={busy} onClick={() => connect(pending.folder, false)}>
              {t.keepLocal}
            </button>
            <button className="btn-secondary" type="button" disabled={busy} onClick={() => setPending(null)}>
              {m.common.cancel}
            </button>
          </div>
        </div>
      ) : connected ? (
        <div className="backup-body">
          <div className="folder-controls-row">
            <input type="text" className="folder-path-field" readOnly value={folder} title={folder} aria-label={t.folderLabel} />
            <button className="btn-secondary" type="button" onClick={() => TauriService.openFolder(folder)} title={m.common.openInExplorer}>
              {m.common.open}
            </button>
          </div>

          <p
            className={`setting-footnote ${status?.last_error ? "error" : ""}`}
            role={status?.last_error ? "alert" : "status"}
          >
            {status?.last_error ?? describeStatus(status, t, (ms) => formatTimeAgo(ms, now))}
          </p>

          <div className="backup-actions">
            <button className="btn-primary" type="button" disabled={busy} onClick={handleSyncNow}>
              {busy ? t.syncing : t.syncNow}
            </button>
            <button className="btn-secondary" type="button" disabled={busy} onClick={handlePickFolder}>
              {t.change}
            </button>
            <button className="btn-secondary" type="button" disabled={busy} onClick={handleDisconnect}>
              {t.turnOff}
            </button>
          </div>
        </div>
      ) : (
        <div className="backup-body">
          <div className="backup-actions">
            <button className="btn-primary" type="button" disabled={busy} onClick={handlePickFolder}>
              {t.choose}
            </button>
          </div>
        </div>
      )}

      <p className="setting-footnote sync-note">{t.note}</p>
    </section>
  );
}
