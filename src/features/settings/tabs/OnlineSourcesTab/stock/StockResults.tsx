import { openUrl } from "@tauri-apps/plugin-opener";
import { DownloadState } from "../../../../../hooks/useWallhavenSearch";
import { StockPage, StockPhoto, StockProviderInfo } from "../../../../../types/stock";
import { useI18n } from "../../../../../i18n/I18nProvider";

interface StockResultsProps {
  info: StockProviderInfo;
  result: StockPage | null;
  loading: boolean;
  downloads: Record<string, DownloadState>;
  onDownload: (photo: StockPhoto) => void;
  onPageChange: (page: number) => void;
}

export function StockResults({ info, result, loading, downloads, onDownload, onPageChange }: StockResultsProps) {
  const { m } = useI18n();
  const t = m.settings.online.stock;

  if (!result) {
    return <p className="results-empty">{loading ? m.common.searching : m.common.noResultsYet}</p>;
  }

  if (result.photos.length === 0) {
    return <p className="results-empty">{t.noMatches}</p>;
  }

  return (
    <section className={`results-section ${loading ? "loading" : ""}`}>
      <div className="contact-sheet-header">
        <span>{m.common.contactSheet}</span>
        <button className="link-button" type="button" onClick={() => openUrl(info.homeUrl)}>
          {t.photosFrom(info.name)}
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
                  {state === "done" ? m.common.saved : state === "downloading" ? m.common.saving : m.common.download}
                </button>
              </div>
              <p className="wallpaper-tile-credit">
                {t.photoBy}{" "}
                <button className="link-button" type="button" onClick={() => openUrl(photo.author_url)}>
                  {photo.author_name}
                </button>{" "}
                {t.on}{" "}
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
          {m.common.previous}
        </button>
        <span>{m.settings.online.page(result.page)}</span>
        <button
          className="btn-secondary"
          type="button"
          disabled={loading || !result.has_next}
          onClick={() => onPageChange(result.page + 1)}
        >
          {m.common.next}
        </button>
      </div>
    </section>
  );
}
