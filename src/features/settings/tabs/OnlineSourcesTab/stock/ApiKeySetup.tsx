import { FormEvent, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { TauriService } from "../../../../../services/tauriCommands";
import { StockProviderInfo } from "../../../../../types/stock";
import { useI18n } from "../../../../../i18n/I18nProvider";

interface ApiKeySetupProps {
  info: StockProviderInfo;
  onSaved: () => void;
  /** Present when replacing an existing key. */
  onCancel?: () => void;
  onFeedback: (type: "success" | "error", message: string) => void;
}

export function ApiKeySetup({ info, onSaved, onCancel, onFeedback }: ApiKeySetupProps) {
  const [key, setKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { m } = useI18n();
  const t = m.settings.online.stock;
  const provider = t.providers[info.id];

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!key.trim()) {
      setError(t.pasteFirst(provider.keyName));
      return;
    }

    try {
      setSaving(true);
      setError(null);
      await TauriService.saveStockApiKey(info.id, key);
      setKey("");
      onFeedback("success", t.connected(info.name));
      onSaved();
    } catch (err) {
      console.error(err);
      setError(typeof err === "string" ? err : t.verifyFailed);
    } finally {
      setSaving(false);
    }
  };

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="7.5" cy="15.5" r="4.5" />
            <path d="M10.7 12.3 21 2" />
            <path d="m16 7 3 3" />
            <path d="m19 4 2 2" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">{t.connect(info.name)}</h3>
          <p className="card-description">{t.connectDescription(info.name)}</p>
        </div>
      </div>

      <div className="card-body">
        <div className="setting-row">
          <span className="setting-label">{t.steps}</span>
          <div className="setting-control">
            <ol className="key-steps">
              {provider.steps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
            <button className="btn-secondary key-link-button" type="button" onClick={() => openUrl(info.keyUrl)}>
              {t.getKey}
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M7 17 17 7" />
                <polyline points="8 7 17 7 17 16" />
              </svg>
            </button>
          </div>
        </div>

        <div className="setting-row">
          <label className="setting-label" htmlFor={`${info.id}-api-key`}>
            {t.keyLabel}
          </label>
          <div className="setting-control">
            <form className="folder-controls-row" onSubmit={handleSubmit}>
              <input
                id={`${info.id}-api-key`}
                type="password"
                className="folder-path-field"
                placeholder={t.pastePlaceholder(provider.keyName)}
                autoComplete="off"
                spellCheck={false}
                value={key}
                disabled={saving}
                onChange={(e) => {
                  setError(null);
                  setKey(e.target.value);
                }}
              />
              <button className="btn-primary" type="submit" disabled={saving}>
                {saving ? t.checking : t.verify}
              </button>
              {onCancel && (
                <button className="btn-secondary" type="button" onClick={onCancel} disabled={saving}>
                  {m.common.cancel}
                </button>
              )}
            </form>
            <p className={`setting-footnote ${error ? "error" : ""}`} role={error ? "alert" : undefined}>
              {error ?? t.verifyNote}
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
