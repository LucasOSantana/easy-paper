import { FormEvent, useEffect, useState } from "react";
import { ToggleSwitch } from "../../components/ToggleSwitch";
import {
  AutoRotateConfig,
  MAX_ROTATION_INTERVAL_MINUTES,
  MIN_ROTATION_INTERVAL_MINUTES,
  RotationOrder,
} from "../../../../types/settings";
import { useI18n } from "../../../../i18n/I18nProvider";

const PRESET_MINUTES = [15, 30, 60, 180, 360, 720, 1440];

type IntervalUnit = "minutes" | "hours";

function splitInterval(minutes: number): { amount: string; unit: IntervalUnit } {
  return minutes % 60 === 0
    ? { amount: String(minutes / 60), unit: "hours" }
    : { amount: String(minutes), unit: "minutes" };
}

interface AutoRotateSectionProps {
  value: AutoRotateConfig;
  favoritesCount: number;
  disabled: boolean;
  onChange: (next: AutoRotateConfig) => Promise<boolean>;
}

export function AutoRotateSection({ value, favoritesCount, disabled, onChange }: AutoRotateSectionProps) {
  const { m } = useI18n();
  const t = m.settings.wallpapers.rotation;

  const orderOptions: { value: RotationOrder; label: string; hint: string }[] = [
    { value: "shuffle", label: t.shuffle, hint: t.shuffleHint },
    { value: "random", label: t.random, hint: t.randomHint },
  ];

  const poolHint = !value.favorites_only
    ? t.poolAll
    : favoritesCount === 0
      ? t.poolNoFavorites
      : t.poolFavorites(favoritesCount);

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
      setCustomError(t.rangeError(MIN_ROTATION_INTERVAL_MINUTES, MAX_ROTATION_INTERVAL_MINUTES / 1440));
      return;
    }

    setCustomError(null);
    await onChange({ ...value, trigger: "interval", interval_minutes: draftMinutes });
  };

  const activeOrder = orderOptions.find((option) => option.value === value.order) ?? orderOptions[0];

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
          <h3 className="card-main-title">{t.title}</h3>
          <p className="card-description">{t.description}</p>
        </div>
        <ToggleSwitch
          checked={value.enabled}
          label={t.toggleLabel}
          disabled={disabled}
          onChange={(enabled) => onChange({ ...value, enabled })}
        />
      </div>

      {value.enabled && (
        <div className="rotation-body">
          <div className="setting-row">
            <label className="setting-label" htmlFor="rotation-schedule">
              {t.changeLabel}
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
                  <option value="on_startup">{t.onStartup}</option>
                  {PRESET_MINUTES.map((minutes) => (
                    <option key={minutes} value={String(minutes)}>
                      {t.preset(minutes)}
                    </option>
                  ))}
                  <option value="custom">{t.custom}</option>
                </select>

                {customOpen && (
                  <form className="custom-interval" onSubmit={handleCustomSubmit}>
                    <span className="custom-interval-prefix">{t.every}</span>
                    <input
                      type="number"
                      className="folder-path-field interval-field"
                      aria-label={t.amountLabel}
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
                      aria-label={t.unitLabel}
                      value={draft.unit}
                      disabled={disabled}
                      onChange={(e) => {
                        setCustomError(null);
                        setDraft({ ...draft, unit: e.target.value as IntervalUnit });
                      }}
                    >
                      <option value="minutes">{t.minutes}</option>
                      <option value="hours">{t.hours}</option>
                    </select>
                    <button
                      className="btn-secondary interval-apply"
                      type="submit"
                      disabled={disabled || draftMatchesSaved}
                    >
                      {draftMatchesSaved ? t.saved : t.apply}
                    </button>
                  </form>
                )}
              </div>

              <p className={`setting-footnote ${customError ? "error" : ""}`} role={customError ? "alert" : undefined}>
                {customError ??
                  (value.trigger === "on_startup" && !customOpen
                    ? t.startupNote
                    : t.intervalNote)}
              </p>
            </div>
          </div>

          <div className="setting-row">
            <span className="setting-label" id="rotation-pool-label">
              {t.fromLabel}
            </span>
            <div className="setting-control">
              <div className="segmented" role="radiogroup" aria-labelledby="rotation-pool-label">
                {[
                  { favoritesOnly: false, label: t.allWallpapers },
                  { favoritesOnly: true, label: t.favoritesOnly },
                ].map((option) => (
                  <label key={String(option.favoritesOnly)} className="segmented-option">
                    <input
                      type="radio"
                      name="rotation-pool"
                      checked={value.favorites_only === option.favoritesOnly}
                      disabled={disabled}
                      onChange={() => onChange({ ...value, favorites_only: option.favoritesOnly })}
                    />
                    <span>{option.label}</span>
                  </label>
                ))}
              </div>
              <p className="setting-footnote">{poolHint}</p>
            </div>
          </div>

          <div className="setting-row">
            <span className="setting-label" id="rotation-order-label">
              {t.orderLabel}
            </span>
            <div className="setting-control">
              <div className="segmented" role="radiogroup" aria-labelledby="rotation-order-label">
                {orderOptions.map((option) => (
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
