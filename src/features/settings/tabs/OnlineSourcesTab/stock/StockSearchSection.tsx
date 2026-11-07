import { StockFilters } from "../../../../../hooks/useStockSearch";
import { StockOrientation, StockProviderInfo } from "../../../../../types/stock";

const ORIENTATION_OPTIONS: { value: StockOrientation; label: string }[] = [
  { value: "landscape", label: "Landscape" },
  { value: "portrait", label: "Portrait" },
  { value: "", label: "Any orientation" },
];

interface StockSearchSectionProps {
  info: StockProviderInfo;
  filters: StockFilters;
  loading: boolean;
  onChange: (changes: Partial<StockFilters>, searchNow: boolean) => void;
  onSearch: () => void;
}

export function StockSearchSection({ info, filters, loading, onChange, onSearch }: StockSearchSectionProps) {
  const browseHint =
    info.id === "unsplash"
      ? "Leave it empty to browse the Unsplash Wallpapers topic."
      : "Leave it empty to browse curated photos (orientation applies to searches only).";

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
          <h3 className="card-main-title">Search</h3>
          <p className="card-description">
            Search photos on {info.name.toLowerCase()}.com. {browseHint}
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
          placeholder="Search keywords..."
          value={filters.query}
          onChange={(e) => onChange({ query: e.target.value }, false)}
        />
        <button className="btn-primary" type="submit" disabled={loading}>
          Search
        </button>
      </form>

      <div className="search-filters-row">
        <select
          className="settings-select"
          aria-label="Orientation"
          value={filters.orientation}
          onChange={(e) => onChange({ orientation: e.target.value as StockOrientation }, true)}
        >
          {ORIENTATION_OPTIONS.map((option) => (
            <option key={option.label} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>
    </section>
  );
}
