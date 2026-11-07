import { FormEvent, useEffect, useState } from "react";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import {
  AutoRotateConfig,
  MAX_ROTATION_INTERVAL_MINUTES,
  MIN_ROTATION_INTERVAL_MINUTES,
  RotationOrder,
} from "../../../../types/settings";

const PRESET_MINUTES = [15, 30, 60, 180, 360, 720, 1440];

const ORDER_OPTIONS: { value: RotationOrder; label: string; hint: string }[] = [
  {
    value: "shuffle",
    label: "Shuffle",
    hint: "Shows every wallpaper in the folder before repeating any.",
  },
  {
    value: "random",
    label: "Random",
    hint: "Picks any wallpaper each time, so some may come back sooner.",
  },
];

type IntervalUnit = "minutes" | "hours";

function presetLabel(minutes: number): string {
  if (minutes < 60) return `Every ${minutes} minutes`;
  if (minutes === 60) return "Every hour";
  if (minutes === 1440) return "Every day";
  return `Every ${minutes / 60} hours`;
}

function splitInterval(minutes: number): { amount: string; unit: IntervalUnit } {
  return minutes % 60 === 0
    ? { amount: String(minutes / 60), unit: "hours" }
    : { amount: String(minutes), unit: "minutes" };
}

interface AutoRotateSectionProps {
  value: AutoRotateConfig;
  disabled: boolean;
  onChange: (next: AutoRotateConfig) => Promise<boolean>;
}

export function AutoRotateSection({ value, disabled, onChange }: AutoRotateSectionProps) {
  const savedIsCustom =
    value.trigger === "interval" && !PRESET_MINUTES.includes(value.interval_minutes);

  const [customOpen, setCustomOpen] = useState(savedIsCustom);
  const [draft, setDraft] = useState(() => splitInterval(value.interval_minutes));
  const [customError, setCustomError] = useState<string | null>(null);

  // Config arrives asynchronously: open the custom editor if the saved value needs it.
  useEffect(() => {
    if (savedIsCustom) {
      setCustomOpen(true);
      setDraft(splitInterval(value.interval_minutes));
    }
  }, [savedIsCustom, value.interval_minutes]);

  const selectValue = customOpen
    ? "custom"
    : value.trigger === "on_startup"
      ? "on_startup"
      : String(value.interval_minutes);

  const draftMinutes = Number(draft.amount) * (draft.unit === "hours" ? 60 : 1);
  const draftMatchesSaved =
    value.trigger === "interval" && draftMinutes === value.interval_minutes;

  const handleScheduleChange = (selected: string) => {
    setCustomError(null);

    if (selected === "custom") {
      setCustomOpen(true);
      setDraft(splitInterval(value.interval_minutes));
      return;
    }

    setCustomOpen(false);
    if (selected === "on_startup") {
      onChange({ ...value, trigger: "on_startup" });
    } else {
      onChange({ ...value, trigger: "interval", interval_minutes: Number(selected) });
    }
  };

  const handleCustomSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (
      !Number.isInteger(draftMinutes) ||
      draftMinutes < MIN_ROTATION_INTERVAL_MINUTES ||
      draftMinutes > MAX_ROTATION_INTERVAL_MINUTES
    ) {
      setCustomError(
        `Choose between ${MIN_ROTATION_INTERVAL_MINUTES} minutes and ${
          MAX_ROTATION_INTERVAL_MINUTES / 1440
        } days.`
      );
      return;
    }

    setCustomError(null);
    await onChange({ ...value, trigger: "interval", interval_minutes: draftMinutes });
  };

  const activeOrder = ORDER_OPTIONS.find((option) => option.value === value.order) ?? ORDER_OPTIONS[0];

  return (
    <section className="feature-card">
      <div className="card-title-row card-title-row-flush">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 12a9 9 0 0 1-15.5 6.2" />
            <path d="M3 12a9 9 0 0 1 15.5-6.2" />
            <polyline points="18.5 2 18.5 6 14.5 6" />
            <polyline points="5.5 22 5.5 18 9.5 18" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">Automatic Rotation</h3>
          <p className="card-description">
            Change the wallpaper on its own, picking from your folder. Each monitor gets a different image.
          </p>
        </div>
        <ToggleSwitch
          checked={value.enabled}
          label="Automatic rotation"
          disabled={disabled}
          onChange={(enabled) => onChange({ ...value, enabled })}
        />
      </div>

      {value.enabled && (
        <div className="rotation-body">
          <div className="setting-row">
            <label className="setting-label" htmlFor="rotation-schedule">
              Change
            </label>
            <div className="setting-control">
              <div className="rotation-schedule-row">
                <select
                  id="rotation-schedule"
                  className="settings-select"
                  value={selectValue}
                  disabled={disabled}
                  onChange={(e) => handleScheduleChange(e.target.value)}
                >
                  <option value="on_startup">Every time Windows starts</option>
                  {PRESET_MINUTES.map((minutes) => (
                    <option key={minutes} value={String(minutes)}>
                      {presetLabel(minutes)}
                    </option>
                  ))}
                  <option value="custom">Custom interval…</option>
                </select>

                {customOpen && (
                  <form className="custom-interval" onSubmit={handleCustomSubmit}>
                    <span className="custom-interval-prefix">Every</span>
                    <input
                      type="number"
                      className="folder-path-field interval-field"
                      aria-label="Interval amount"
                      min={1}
                      step={1}
                      value={draft.amount}
                      disabled={disabled}
                      onChange={(e) => {
                        setCustomError(null);
                        setDraft({ ...draft, amount: e.target.value });
                      }}
                    />
                    <select
                      className="settings-select"
                      aria-label="Interval unit"
                      value={draft.unit}
                      disabled={disabled}
                      onChange={(e) => {
                        setCustomError(null);
                        setDraft({ ...draft, unit: e.target.value as IntervalUnit });
                      }}
                    >
                      <option value="minutes">minutes</option>
                      <option value="hours">hours</option>
                    </select>
                    <button
                      className="btn-secondary interval-apply"
                      type="submit"
                      disabled={disabled || draftMatchesSaved}
                    >
                      {draftMatchesSaved ? "Saved" : "Apply"}
                    </button>
                  </form>
                )}
              </div>

              <p className={`setting-footnote ${customError ? "error" : ""}`} role={customError ? "alert" : undefined}>
                {customError ??
                  (value.trigger === "on_startup" && !customOpen
                    ? "Changes once each time you sign in to Windows after turning the PC on or restarting."
                    : "Time asleep counts too, so it changes on wake if the interval has passed. Starts over when the PC is turned on or you pick a wallpaper yourself.")}
              </p>
            </div>
          </div>

          <div className="setting-row">
            <span className="setting-label" id="rotation-order-label">
              Order
            </span>
            <div className="setting-control">
              <div className="segmented" role="radiogroup" aria-labelledby="rotation-order-label">
                {ORDER_OPTIONS.map((option) => (
                  <label key={option.value} className="segmented-option">
                    <input
                      type="radio"
                      name="rotation-order"
                      value={option.value}
                      checked={value.order === option.value}
                      disabled={disabled}
                      onChange={() => onChange({ ...value, order: option.value })}
                    />
                    <span>{option.label}</span>
                  </label>
                ))}
              </div>
              <p className="setting-footnote">{activeOrder.hint}</p>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
