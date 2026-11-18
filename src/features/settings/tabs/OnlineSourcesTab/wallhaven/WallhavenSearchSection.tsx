import { ToggleSwitch } from "../../../components/ToggleSwitch";
import { WallhavenFilters } from "../../../../../hooks/useWallhavenSearch";
import { WallhavenSorting } from "../../../../../types/wallhaven";
import { useI18n } from "../../../../../i18n/I18nProvider";

const RATIO_OPTIONS = [
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
  const { m } = useI18n();
  const t = m.settings.online.wallhaven;
  const sortingOptions: { value: WallhavenSorting; label: string }[] = [
    { value: "toplist", label: t.sortTop },
    { value: "relevance", label: t.sortRelevance },
    { value: "date_added", label: t.sortLatest },
    { value: "views", label: t.sortViews },
    { value: "favorites", label: t.sortFavorites },
    { value: "random", label: t.sortRandom },
  ];
  const ratioOptions = [{ value: "", label: t.anyRatio }, ...RATIO_OPTIONS];

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
          <p className="card-description">{t.description}</p>
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
          value={filters.sorting}
          onChange={(e) => onChange({ sorting: e.target.value as WallhavenSorting }, true)}
        >
          {sortingOptions.map((option) => (
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
          {ratioOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>

        <label className="search-toggle-label">
          <ToggleSwitch
            checked={filters.match_screen}
            label={t.matchScreen}
            onChange={(matchScreen) => onChange({ match_screen: matchScreen }, true)}
          />
          <span>{t.matchScreen}</span>
        </label>
      </div>
    </section>
  );
}
