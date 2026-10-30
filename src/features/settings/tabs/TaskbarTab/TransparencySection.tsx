import { ToggleSwitch } from "../../components/ToggleSwitch";

interface TransparencySectionProps {
  transparent: boolean;
  disabled: boolean;
  onChange: (transparent: boolean) => void;
}

export function TransparencySection({
  transparent,
  disabled,
  onChange,
}: TransparencySectionProps) {
  return (
    <section className="feature-card">
      <div className="card-title-row card-title-row-flush">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="2" y="15" width="20" height="6" rx="1.5" strokeDasharray="3 2" />
            <path d="M6 3h12M8 7h8M10 11h4" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">Transparent Taskbar</h3>
          <p className="card-description">
            Remove the taskbar background so your wallpaper shows through. Icons stay visible.
          </p>
        </div>
        <ToggleSwitch
          checked={transparent}
          label="Transparent taskbar"
          disabled={disabled}
          onChange={onChange}
        />
      </div>
    </section>
  );
}
