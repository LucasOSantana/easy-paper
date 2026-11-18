import { useState } from "react";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import { TauriService } from "../../../../services/tauriCommands";
import { BackupSummary } from "../../../../types/sync";
import { useI18n } from "../../../../i18n/I18nProvider";

interface BackupSectionProps {
  onFeedback: (type: "success" | "error", message: string) => void;
}

type LastResult = { kind: "export" | "import"; summary: BackupSummary };

export function BackupSection({ onFeedback }: BackupSectionProps) {
  const [includeImages, setIncludeImages] = useState(true);
  const [busy, setBusy] = useState<"export" | "import" | null>(null);
  const [lastResult, setLastResult] = useState<LastResult | null>(null);
  const { m } = useI18n();
  const t = m.settings.backup;

  const handleExport = async () => {
    try {
      setBusy("export");
      const summary = await TauriService.exportBackup(includeImages);
      if (!summary) return;
      setLastResult({ kind: "export", summary });
      onFeedback("success", t.saved);
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : t.saveFailed);
    } finally {
      setBusy(null);
    }
  };

  const handleImport = async () => {
    try {
      setBusy("import");
      const summary = await TauriService.importBackup();
      if (!summary) return;
      setLastResult({ kind: "import", summary });
      onFeedback(
        summary.warnings.length > 0 ? "error" : "success",
        summary.warnings.length > 0 ? t.restoredWithWarnings : t.restored
      );
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : t.restoreFailed);
    } finally {
      setBusy(null);
    }
  };

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">{t.cardTitle}</h3>
          <p className="card-description">{t.cardDescription}</p>
        </div>
      </div>

      <div className="backup-body">
        <div className="backup-option">
          <div className="backup-option-text">
            <span className="backup-option-title">{t.includeImages}</span>
            <span className="setting-footnote">{t.includeImagesNote}</span>
          </div>
          <ToggleSwitch
            checked={includeImages}
            label={t.includeImages}
            disabled={busy !== null}
            onChange={setIncludeImages}
          />
        </div>

        <div className="backup-actions">
          <button className="btn-primary" type="button" onClick={handleExport} disabled={busy !== null}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            <span>{busy === "export" ? t.exporting : t.export}</span>
          </button>
          <button className="btn-secondary" type="button" onClick={handleImport} disabled={busy !== null}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            <span>{busy === "import" ? t.restoring : t.restore}</span>
          </button>
        </div>

        <p className="setting-footnote">{t.restoreNote}</p>

        {lastResult && (
          <div className="backup-result" role="status">
            <div className="backup-result-text">
              <span className="backup-result-title">
                {lastResult.kind === "export"
                  ? t.savedSummary(lastResult.summary.images)
                  : t.restoredSummary(lastResult.summary.images)}
              </span>
              <span className="backup-result-path" title={lastResult.summary.path}>
                {lastResult.summary.path}
              </span>
              {lastResult.summary.warnings.map((warning) => (
                <span key={warning} className="setting-footnote error">
                  {warning}
                </span>
              ))}
            </div>
            {lastResult.kind === "export" && (
              <button
                className="btn-secondary backup-result-open"
                type="button"
                onClick={() => TauriService.openFolder(lastResult.summary.path)}
              >
                {m.common.open}
              </button>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
