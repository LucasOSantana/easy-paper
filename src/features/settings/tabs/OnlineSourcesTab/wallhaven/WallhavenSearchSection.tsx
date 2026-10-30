import { ToggleSwitch } from "../../../components/ToggleSwitch";
import { WallhavenFilters } from "../../../../../hooks/useWallhavenSearch";
import { WallhavenSorting } from "../../../../../types/wallhaven";

const SORTING_OPTIONS: { value: WallhavenSorting; label: string }[] = [
  { value: "toplist", label: "Top" },
  { value: "relevance", label: "Relevance" },
  { value: "date_added", label: "Latest" },
  { value: "views", label: "Most viewed" },
  { value: "favorites", label: "Most favorited" },
  { value: "random", label: "Random" },
];

const RATIO_OPTIONS = [
  { value: "", label: "Any ratio" },
  { value: "16x9", label: "16:9" },
  { value: "16x10", label: "16:10" },
  { value: "21x9", label: "21:9" },
  { value: "32x9", label: "32:9" },
  { value: "4x3", label: "4:3" },
  { value: "9x16", label: "9:16" },
];

interface WallhavenSearchSectionProps {
  filters: WallhavenFilters;
  loading: boolean;
  onChange: (changes: Partial<WallhavenFilters>, searchNow: boolean) => void;
  onSearch: () => void;
}

export function WallhavenSearchSection({
  filters,
  loading,
  onChange,
  onSearch,
}: WallhavenSearchSectionProps) {
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
            Search safe-for-work wallpapers from wallhaven.cc.
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
          placeholder="Search tags or keywords..."
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
          value={filters.sorting}
          onChange={(e) => onChange({ sorting: e.target.value as WallhavenSorting }, true)}
        >
          {SORTING_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>

        <select
          className="settings-select"
          value={filters.ratios}
          onChange={(e) => onChange({ ratios: e.target.value }, true)}
        >
          {RATIO_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>

        <label className="search-toggle-label">
          <ToggleSwitch
            checked={filters.match_screen}
            label="At least my screen resolution"
            onChange={(matchScreen) => onChange({ match_screen: matchScreen }, true)}
          />
          <span>At least my screen resolution</span>
        </label>
      </div>
    </section>
  );
}
