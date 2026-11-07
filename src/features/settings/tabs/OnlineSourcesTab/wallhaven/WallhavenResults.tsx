import { DownloadState } from "../../../../../hooks/useWallhavenSearch";
import { WallhavenPage, WallhavenWallpaper } from "../../../../../types/wallhaven";

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
  if (!result) {
    return <p className="results-empty">{loading ? "Searching..." : "No results yet."}</p>;
  }

  if (result.data.length === 0) {
    return <p className="results-empty">No wallpapers match these filters.</p>;
  }

  const { current_page: page, last_page: lastPage } = result.meta;

  return (
    <section className={`results-section ${loading ? "loading" : ""}`}>
      <div className="contact-sheet-header">
        <span>Contact sheet</span>
        <span className="contact-sheet-count">{result.data.length} frames</span>
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
                  {state === "done" ? "Saved" : state === "downloading" ? "Saving..." : "Download"}
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
          Previous
        </button>
        <span>
          Page {page} of {lastPage}
        </span>
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || page >= lastPage}
          onClick={() => onPageChange(page + 1)}
        >
          Next
        </button>
      </div>
    </section>
  );
}
