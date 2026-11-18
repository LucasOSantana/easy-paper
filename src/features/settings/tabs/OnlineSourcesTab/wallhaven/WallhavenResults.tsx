import { DownloadState } from "../../../../../hooks/useWallhavenSearch";
import { WallhavenPage, WallhavenWallpaper } from "../../../../../types/wallhaven";
import { useI18n } from "../../../../../i18n/I18nProvider";

interface WallhavenResultsProps {
  result: WallhavenPage | null;
  loading: boolean;
  downloads: Record<string, DownloadState>;
  onDownload: (wallpaper: WallhavenWallpaper) => void;
  onPageChange: (page: number) => void;
}

export function WallhavenResults({
  result,
  loading,
  downloads,
  onDownload,
  onPageChange,
}: WallhavenResultsProps) {
  const { m } = useI18n();

  if (!result) {
    return <p className="results-empty">{loading ? m.common.searching : m.common.noResultsYet}</p>;
  }

  if (result.data.length === 0) {
    return <p className="results-empty">{m.settings.online.wallhaven.noMatches}</p>;
  }

  const { current_page: page, last_page: lastPage } = result.meta;

  return (
    <section className={`results-section ${loading ? "loading" : ""}`}>
      <div className="contact-sheet-header">
        <span>{m.common.contactSheet}</span>
        <span className="contact-sheet-count">{m.settings.online.frames(result.data.length)}</span>
      </div>

      <div className="wallpaper-grid">
        {result.data.map((wallpaper, index) => {
          const state = downloads[wallpaper.id];
          return (
            <div key={wallpaper.id} className="wallpaper-tile">
              <div className="wallpaper-tile-image">
                <img src={wallpaper.thumbs.small} alt="" loading="lazy" decoding="async" />
              </div>
              <div className="wallpaper-tile-bar">
                <span className="wallpaper-tile-meta">
                  <span className="wallpaper-tile-number">▸ {String(index + 1).padStart(2, "0")}</span>
                  <span>{wallpaper.resolution}</span>
                </span>
                <button
                  className="btn-secondary wallpaper-tile-button"
                  type="button"
                  disabled={state !== undefined}
                  onClick={() => onDownload(wallpaper)}
                >
                  {state === "done" ? m.common.saved : state === "downloading" ? m.common.saving : m.common.download}
                </button>
              </div>
            </div>
          );
        })}
      </div>

      <div className="pagination-row">
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || page <= 1}
          onClick={() => onPageChange(page - 1)}
        >
          {m.common.previous}
        </button>
        <span>{m.settings.online.pageOf(page, lastPage)}</span>
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || page >= lastPage}
          onClick={() => onPageChange(page + 1)}
        >
          {m.common.next}
        </button>
      </div>
    </section>
  );
}
