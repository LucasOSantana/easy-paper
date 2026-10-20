import { useShortcutRecorder } from "../../../../hooks/useShortcutRecorder";

interface ShortcutSectionProps {
  currentShortcut: string;
  onUpdateShortcut: (shortcut: string) => Promise<boolean>;
}

export function ShortcutSection({
  currentShortcut,
  onUpdateShortcut,
}: ShortcutSectionProps) {
  const { isRecording, previewKeys, startRecording, cancelRecording } =
    useShortcutRecorder({
      onShortcutCaptured: onUpdateShortcut,
    });

  const renderKeycaps = (shortcutStr: string) => {
    if (!shortcutStr) return null;
    const tokens = shortcutStr.split("+");
    return tokens.map((token, idx) => (
      <span key={`${token}-${idx}`} style={{ display: "inline-flex", alignItems: "center", gap: "6px" }}>
        {idx > 0 && <span className="keycap-separator">+</span>}
        <kbd className="keycap-badge">{token}</kbd>
      </span>
    ));
  };

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="2" y="4" width="20" height="16" rx="2" />
            <line x1="6" y1="8" x2="6" y2="8" />
            <line x1="10" y1="8" x2="10" y2="8" />
            <line x1="14" y1="8" x2="14" y2="8" />
            <line x1="18" y1="8" x2="18" y2="8" />
            <line x1="6" y1="12" x2="6" y2="12" />
            <line x1="18" y1="12" x2="18" y2="12" />
            <line x1="10" y1="16" x2="14" y2="16" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">Global Invocation Shortcut</h3>
          <p className="card-description">
            Press this key combination from anywhere in Windows to toggle the wallpaper carousel.
          </p>
        </div>
      </div>

      <div className="shortcut-display-box">
        {isRecording ? (
          <div className="recording-active-banner">
            <span className="recording-pulse-point" />
            <span>
              {previewKeys.length > 0
                ? previewKeys.join(" + ")
                : "Press desired keys on your keyboard... (Esc to cancel)"}
            </span>
          </div>
        ) : (
          <div className="keycaps-cluster">{renderKeycaps(currentShortcut)}</div>
        )}

        <div>
          {isRecording ? (
            <button className="btn-secondary" onClick={cancelRecording} type="button">
              Cancel
            </button>
          ) : (
            <button className="btn-primary" onClick={startRecording} type="button">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
              </svg>
              <span>Record Shortcut</span>
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
