import { StockFilters } from "../../../../../hooks/useStockSearch";
import { StockOrientation, StockProviderInfo } from "../../../../../types/stock";
import { useI18n } from "../../../../../i18n/I18nProvider";

interface StockSearchSectionProps {
  info: StockProviderInfo;
  filters: StockFilters;
  loading: boolean;
  onChange: (changes: Partial<StockFilters>, searchNow: boolean) => void;
  onSearch: () => void;
}

export function StockSearchSection({ info, filters, loading, onChange, onSearch }: StockSearchSectionProps) {
  const { m } = useI18n();
  const t = m.settings.online.stock;
  const browseHint = info.id === "unsplash" ? t.browseUnsplash : t.browsePexels;
  const orientationOptions: { value: StockOrientation; label: string }[] = [
    { value: "landscape", label: t.landscape },
    { value: "portrait", label: t.portrait },
    { value: "", label: t.anyOrientation },
  ];

  return (
    <section className="feature-card">
      <div className="card-title-row">
        <div className="card-icon-badge">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="7" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
        </div>
        <div className="card-headings">
          <h3 className="card-main-title">{m.common.search}</h3>
          <p className="card-description">
            {t.description(`${info.name.toLowerCase()}.com`)} {browseHint}
          </p>
        </div>
      </div>

      <form
        className="folder-controls-row"
        onSubmit={(e) => {
          e.preventDefault();
          onSearch();
        }}
      >
        <input
          type="search"
          className="folder-path-field search-query-field"
          placeholder={t.placeholder}
          value={filters.query}
          onChange={(e) => onChange({ query: e.target.value }, false)}
        />
        <button className="btn-primary" type="submit" disabled={loading}>
          {m.common.search}
        </button>
      </form>

      <div className="search-filters-row">
        <select
          className="settings-select"
          aria-label={t.orientationLabel}
          value={filters.orientation}
          onChange={(e) => onChange({ orientation: e.target.value as StockOrientation }, true)}
        >
          {orientationOptions.map((option) => (
            <option key={option.value || "any"} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>
    </section>
  );
}
