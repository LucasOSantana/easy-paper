import { useI18n } from "../../i18n/I18nProvider";

export function ShortcutLegend({ onClose }: { onClose: () => void }) {
  const { m } = useI18n();
  const legend = m.carousel.legend;

  const shortcuts: { keys: string[]; label: string }[] = [
    { keys: ["←", "→"], label: legend.browse },
    { keys: ["Home", "End"], label: legend.firstLast },
    { keys: ["Enter"], label: legend.apply },
    { keys: ["/"], label: legend.search },
    { keys: ["Tab"], label: legend.toggleView },
    { keys: ["F"], label: legend.favorite },
    { keys: ["R"], label: legend.random },
    { keys: ["Del"], label: legend.delete },
    { keys: ["Ctrl", "O"], label: legend.explorer },
    { keys: ["Ctrl", ","], label: legend.settings },
    { keys: ["Esc"], label: legend.escape },
  ];

  return (
    <div className="shortcut-legend" role="dialog" aria-label={m.carousel.shortcutsLabel}>
      <div className="shortcut-legend-header">
        <span className="shortcut-legend-title">{legend.title}</span>
        <button className="shortcut-legend-close" type="button" onClick={onClose} aria-label={legend.close}>
          ×
        </button>
      </div>
      <dl className="shortcut-legend-list">
        {shortcuts.map((shortcut) => (
          <div key={shortcut.keys.join("+")} className="shortcut-legend-row">
            <dt>
              {shortcut.keys.map((key) => (
                <kbd key={key}>{key}</kbd>
              ))}
            </dt>
            <dd>{shortcut.label}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
