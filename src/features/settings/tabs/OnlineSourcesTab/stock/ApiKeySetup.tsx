import { FormEvent, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { TauriService } from "../../../../../services/tauriCommands";
import { StockProviderInfo } from "../../../../../types/stock";

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

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!key.trim()) {
      setError(`Paste your ${info.keyLabel} first.`);
      return;
    }

    try {
      setSaving(true);
      setError(null);
      await TauriService.saveStockApiKey(info.id, key);
      setKey("");
      onFeedback("success", `${info.name} connected.`);
      onSaved();
    } catch (err) {
      console.error(err);
      setError(typeof err === "string" ? err : "Could not verify this key.");
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
          <h3 className="card-main-title">Connect {info.name}</h3>
          <p className="card-description">
            {info.name} needs a free personal API key. Easy Paper keeps it in Windows Credential Manager on this PC
            and only sends it to {info.name}.
          </p>
        </div>
      </div>

      <div className="card-body">
        <div className="setting-row">
          <span className="setting-label">Steps</span>
          <div className="setting-control">
            <ol className="key-steps">
              {info.keySteps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
            <button className="btn-secondary key-link-button" type="button" onClick={() => openUrl(info.keyUrl)}>
              Get a free key
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <path d="M7 17 17 7" />
                <polyline points="8 7 17 7 17 16" />
              </svg>
            </button>
          </div>
        </div>

        <div className="setting-row">
          <label className="setting-label" htmlFor={`${info.id}-api-key`}>
            Key
          </label>
          <div className="setting-control">
            <form className="folder-controls-row" onSubmit={handleSubmit}>
              <input
                id={`${info.id}-api-key`}
                type="password"
                className="folder-path-field"
                placeholder={`Paste your ${info.keyLabel}`}
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
                {saving ? "Checking…" : "Verify & Save"}
              </button>
              {onCancel && (
                <button className="btn-secondary" type="button" onClick={onCancel} disabled={saving}>
                  Cancel
                </button>
              )}
            </form>
            <p className={`setting-footnote ${error ? "error" : ""}`} role={error ? "alert" : undefined}>
              {error ?? "The key is checked with a test request before it is saved."}
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
