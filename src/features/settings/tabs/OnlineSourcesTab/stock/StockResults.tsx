import { openUrl } from "@tauri-apps/plugin-opener";
import { DownloadState } from "../../../../../hooks/useWallhavenSearch";
import { StockPage, StockPhoto, StockProviderInfo } from "../../../../../types/stock";

interface StockResultsProps {
  info: StockProviderInfo;
  result: StockPage | null;
  loading: boolean;
  downloads: Record<string, DownloadState>;
  onDownload: (photo: StockPhoto) => void;
  onPageChange: (page: number) => void;
}

export function StockResults({ info, result, loading, downloads, onDownload, onPageChange }: StockResultsProps) {
  if (!result) {
    return <p className="results-empty">{loading ? "Searching..." : "No results yet."}</p>;
  }

  if (result.photos.length === 0) {
    return <p className="results-empty">No photos match this search.</p>;
  }

  return (
    <section className={`results-section ${loading ? "loading" : ""}`}>
      <div className="contact-sheet-header">
        <span>Contact sheet</span>
        <button className="link-button" type="button" onClick={() => openUrl(info.homeUrl)}>
          Photos from {info.name} ↗
        </button>
      </div>

      <div className="wallpaper-grid">
        {result.photos.map((photo, index) => {
          const state = downloads[photo.id];
          return (
            <div key={photo.id} className="wallpaper-tile">
              <div className="wallpaper-tile-image">
                <img src={photo.thumb_url} alt="" loading="lazy" decoding="async" />
              </div>
              <div className="wallpaper-tile-bar">
                <span className="wallpaper-tile-meta">
                  <span className="wallpaper-tile-number">▸ {String(index + 1).padStart(2, "0")}</span>
                  <span>
                    {photo.width}x{photo.height}
                  </span>
                </span>
                <button
                  className="btn-secondary wallpaper-tile-button"
                  type="button"
                  disabled={state !== undefined}
                  onClick={() => onDownload(photo)}
                >
                  {state === "done" ? "Saved" : state === "downloading" ? "Saving..." : "Download"}
                </button>
              </div>
              <p className="wallpaper-tile-credit">
                Photo by{" "}
                <button className="link-button" type="button" onClick={() => openUrl(photo.author_url)}>
                  {photo.author_name}
                </button>{" "}
                on{" "}
                <button className="link-button" type="button" onClick={() => openUrl(photo.page_url)}>
                  {info.name}
                </button>
              </p>
            </div>
          );
        })}
      </div>

      <div className="pagination-row">
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || result.page <= 1}
          onClick={() => onPageChange(result.page - 1)}
        >
          Previous
        </button>
        <span>Page {result.page}</span>
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || !result.has_next}
          onClick={() => onPageChange(result.page + 1)}
        >
          Next
        </button>
      </div>
    </section>
  );
}
