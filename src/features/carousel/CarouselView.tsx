import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ImageItem } from "../../types/wallpaper";
import { TauriService } from "../../services/tauriCommands";
import { filterLibrary } from "./librarySearch";
import { ShortcutLegend } from "./ShortcutLegend";
import { useI18n } from "../../i18n/I18nProvider";

const FAVORITES_VIEW_KEY = "easy-paper:favorites-view";
const NOTICE_MS = 2600;

function readFavoritesView(): boolean {
  try {
    return localStorage.getItem(FAVORITES_VIEW_KEY) === "1";
  } catch {
    return false;
  }
}

function saveFavoritesView(on: boolean) {
  try {
    localStorage.setItem(FAVORITES_VIEW_KEY, on ? "1" : "0");
  } catch {
    // Only a convenience: ignore when storage is unavailable.
  }
}

type Notice = { tone: "info" | "error"; text: string };

function errorText(err: unknown, fallback: string): string {
  return typeof err === "string" ? err : fallback;
}

function StarIcon({ filled, size = 14 }: { filled: boolean; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke="currentColor" strokeWidth="1.75" aria-hidden="true">
      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
    </svg>
  );
}

export function CarouselView() {
  const [images, setImages] = useState<ImageItem[]>([]);
  const [loading, setLoading] = useState(true);
  /** The focused wallpaper. Kept by path so it survives filtering and reloads. */
  const [activePath, setActivePath] = useState<string | null>(null);
  const [applyingPath, setApplyingPath] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [favoritesView, setFavoritesView] = useState(readFavoritesView);
  const [pendingDelete, setPendingDelete] = useState<ImageItem | null>(null);
  const [showShortcuts, setShowShortcuts] = useState(false);
  const [notice, setNotice] = useState<Notice | null>(null);
  const { m } = useI18n();
  const t = m.carousel;

  const isScrollingRef = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);
  const noticeTimer = useRef<number | undefined>(undefined);

  const visible = useMemo(() => filterLibrary(images, query, favoritesView), [images, query, favoritesView]);
  const foundIndex = visible.findIndex((img) => img.path === activePath);
  const activeIndex = foundIndex >= 0 ? foundIndex : 0;
  const active: ImageItem | undefined = visible[activeIndex];

  const showNotice = (tone: Notice["tone"], text: string) => {
    window.clearTimeout(noticeTimer.current);
    setNotice({ tone, text });
    noticeTimer.current = window.setTimeout(() => setNotice(null), NOTICE_MS);
  };

  async function loadImages() {
    try {
      setImages(await TauriService.getImages());
    } catch (error) {
      console.error("Failed to load wallpapers:", error);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadImages();

    // The window is reused: each time it is summoned, start from a clean slate.
    const handleFocus = () => {
      (document.activeElement as HTMLElement | null)?.blur();
      setQuery("");
      setPendingDelete(null);
      setShowShortcuts(false);
      loadImages();
    };

    window.addEventListener("focus", handleFocus);
    return () => {
      window.removeEventListener("focus", handleFocus);
      window.clearTimeout(noticeTimer.current);
    };
  }, []);

  const focusIndex = (index: number) => {
    if (visible.length === 0) return;
    const clamped = Math.min(Math.max(index, 0), visible.length - 1);
    setActivePath(visible[clamped].path);
  };

  /** The wallpaper that takes the focus when the active one leaves the view. */
  const neighborOf = (index: number) => visible[index + 1] ?? visible[index - 1] ?? null;

  const handleOpenSettings = async () => {
    try {
      await TauriService.openSettings();
    } catch (err) {
      console.error("Failed to open settings window:", err);
    }
  };

  const hideWindow = async () => {
    try {
      await TauriService.hideMainWindow();
    } catch (err) {
      console.error("Failed to hide window:", err);
    }
  };

  const handleSelect = async (img: ImageItem) => {
    if (applyingPath) return;

    setActivePath(img.path);
    setApplyingPath(img.path);

    try {
      await TauriService.setWallpaper(img.path);
    } catch (error) {
      console.error("Failed to apply desktop wallpaper:", error);
    }

    setTimeout(async () => {
      await hideWindow();
      setApplyingPath(null);
    }, 650);
  };

  const toggleFavorite = async (img: ImageItem) => {
    const next = !img.favorite;
    if (!next && favoritesView && img.path === active?.path) {
      // It is about to leave the favorites view: keep the focus nearby.
      setActivePath(neighborOf(activeIndex)?.path ?? null);
    }

    const setFlag = (favorite: boolean) =>
      setImages((list) => list.map((item) => (item.path === img.path ? { ...item, favorite } : item)));

    setFlag(next);
    try {
      await TauriService.setFavorite(img.path, next);
      showNotice("info", next ? t.addedToFavorites : t.removedFromFavorites);
    } catch (err) {
      setFlag(!next);
      showNotice("error", errorText(err, t.favoriteFailed));
    }
  };

  const toggleFavoritesView = () => {
    setFavoritesView((on) => {
      saveFavoritesView(!on);
      return !on;
    });
  };

  const confirmDelete = async () => {
    const target = pendingDelete;
    if (!target) return;
    setPendingDelete(null);

    const index = visible.findIndex((img) => img.path === target.path);
    try {
      await TauriService.deleteWallpaper(target.path);
      if (index >= 0) setActivePath(neighborOf(index)?.path ?? null);
      setImages((list) => list.filter((item) => item.path !== target.path));
      showNotice("info", t.movedToRecycleBin(target.name));
    } catch (err) {
      showNotice("error", errorText(err, t.deleteFailed));
    }
  };

  const focusRandom = () => {
    if (visible.length < 2) return;
    let index = activeIndex;
    while (index === activeIndex) {
      index = Math.floor(Math.random() * visible.length);
    }
    focusIndex(index);
  };

  const reveal = async (img: ImageItem) => {
    try {
      await TauriService.revealWallpaper(img.path);
    } catch (err) {
      showNotice("error", errorText(err, t.explorerFailed));
    }
  };

  const openSearch = () => {
    setShowShortcuts(false);
    searchRef.current?.focus();
    searchRef.current?.select();
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    const ctrl = e.ctrlKey || e.metaKey;

    if (ctrl && e.key === ",") {
      e.preventDefault();
      handleOpenSettings();
      return;
    }
    if (applyingPath) return;

    if (pendingDelete) {
      e.preventDefault();
      if (e.key === "Enter" || e.key === "Delete") confirmDelete();
      else if (e.key === "Escape") setPendingDelete(null);
      return;
    }

    const inSearch = e.target === searchRef.current;

    // Keys that work everywhere, including while typing a search.
    switch (e.key) {
      case "ArrowRight":
        e.preventDefault();
        focusIndex(activeIndex + 1);
        return;
      case "ArrowLeft":
        e.preventDefault();
        focusIndex(activeIndex - 1);
        return;
      case "Enter":
        e.preventDefault();
        if (active) handleSelect(active);
        return;
      case "Tab":
        e.preventDefault();
        toggleFavoritesView();
        return;
      case "Escape":
        e.preventDefault();
        if (showShortcuts) setShowShortcuts(false);
        else if (query) setQuery("");
        else if (inSearch) searchRef.current?.blur();
        else hideWindow();
        return;
    }

    if (ctrl && e.key.toLowerCase() === "f") {
      e.preventDefault();
      openSearch();
      return;
    }
    if (ctrl && e.key.toLowerCase() === "o") {
      e.preventDefault();
      if (active) reveal(active);
      return;
    }

    // Everything below is a single key, which would type into the search.
    if (inSearch || ctrl || e.altKey) return;

    switch (e.key) {
      case "/":
        e.preventDefault();
        openSearch();
        break;
      case "?":
        setShowShortcuts((on) => !on);
        break;
      case "Home":
        e.preventDefault();
        focusIndex(0);
        break;
      case "End":
        e.preventDefault();
        focusIndex(visible.length - 1);
        break;
      case "PageDown":
        e.preventDefault();
        focusIndex(activeIndex + 5);
        break;
      case "PageUp":
        e.preventDefault();
        focusIndex(activeIndex - 5);
        break;
      case " ":
        e.preventDefault();
        if (active) handleSelect(active);
        break;
      case "f":
      case "F":
        if (active) toggleFavorite(active);
        break;
      case "r":
      case "R":
        focusRandom();
        break;
      case "Delete":
        if (active) setPendingDelete(active);
        break;
    }
  };

  // Always run the latest handler without re-subscribing on every render.
  const keyHandlerRef = useRef(handleKeyDown);
  keyHandlerRef.current = handleKeyDown;
  useEffect(() => {
    const listener = (e: KeyboardEvent) => keyHandlerRef.current(e);
    window.addEventListener("keydown", listener);
    return () => window.removeEventListener("keydown", listener);
  }, []);

  const handleWheel = (e: React.WheelEvent) => {
    if (visible.length === 0 || isScrollingRef.current || applyingPath || pendingDelete) return;
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(delta) < 20) return;

    isScrollingRef.current = true;
    setTimeout(() => {
      isScrollingRef.current = false;
    }, 150);

    focusIndex(activeIndex + (delta > 0 ? 1 : -1));
  };

  const favoritesCount = images.filter((img) => img.favorite).length;

  return (
    <div className="carousel-container" onWheel={handleWheel}>
      {images.length > 0 && (
        <div className="carousel-toolbar">
          <label className={`carousel-search ${query ? "has-query" : ""}`}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" aria-hidden="true">
              <circle cx="11" cy="11" r="7" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              ref={searchRef}
              type="text"
              value={query}
              placeholder={t.searchPlaceholder}
              aria-label={t.searchLabel}
              spellCheck={false}
              onChange={(e) => setQuery(e.target.value)}
            />
            {query ? (
              <button type="button" className="carousel-search-clear" onClick={() => setQuery("")} aria-label={t.clearSearch}>
                ×
              </button>
            ) : (
              <kbd className="btn-settings-hint">/</kbd>
            )}
          </label>

          <div className="carousel-filter" role="radiogroup" aria-label={t.filterLabel}>
            <button
              type="button"
              role="radio"
              aria-checked={!favoritesView}
              className={!favoritesView ? "on" : ""}
              onClick={() => favoritesView && toggleFavoritesView()}
            >
              {t.filterAll} <span className="carousel-filter-count">{images.length}</span>
            </button>
            <button
              type="button"
              role="radio"
              aria-checked={favoritesView}
              className={favoritesView ? "on" : ""}
              onClick={() => !favoritesView && toggleFavoritesView()}
            >
              <StarIcon filled={favoritesView} size={12} />
              {t.filterFavorites} <span className="carousel-filter-count">{favoritesCount}</span>
            </button>
          </div>
        </div>
      )}

      <div className="carousel-topbar">
        <button
          className="btn-settings-trigger btn-icon-only"
          onClick={() => setShowShortcuts((on) => !on)}
          title={t.shortcutsTitle}
          aria-label={t.shortcutsLabel}
          aria-expanded={showShortcuts}
        >
          ?
        </button>
        <button
          className="btn-settings-trigger"
          onClick={handleOpenSettings}
          title={t.settingsTitle}
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
          <span>{t.settings}</span>
          <kbd className="btn-settings-hint">Ctrl ,</kbd>
        </button>
      </div>

      {showShortcuts && <ShortcutLegend onClose={() => setShowShortcuts(false)} />}

      {loading && images.length === 0 ? (
        <div className="carousel-empty">
          <span className="carousel-empty-eyebrow">{t.loading}</span>
        </div>
      ) : images.length === 0 ? (
        <div className="carousel-empty">
          <span className="carousel-empty-eyebrow">{t.emptyEyebrow}</span>
          <p>{t.emptyText}</p>
          <button className="btn-settings-trigger" onClick={handleOpenSettings}>
            {t.configureFolder}
          </button>
        </div>
      ) : visible.length === 0 ? (
        <div className="carousel-empty">
          <span className="carousel-empty-eyebrow">{query ? t.noMatchesEyebrow : t.noFavoritesEyebrow}</span>
          <p>{query ? t.noMatches(query, favoritesView) : t.noFavorites}</p>
          <span className="carousel-empty-hint">{query ? t.escClearsSearch : t.tabShowsAll}</span>
        </div>
      ) : (
        <div className="carousel-stage">
          <div className="carousel-track">
            {visible.map((img, index) => {
              const diff = index - activeIndex;
              const absDiff = Math.abs(diff);

              if (absDiff > 8) return null;

              const xOffset = diff * 140;
              const rotateY = diff === 0 ? 0 : diff > 0 ? -38 : 38;
              const translateZ = diff === 0 ? 90 : -absDiff * 45;
              const scale = diff === 0 ? 1 : Math.max(0.74, 1 - absDiff * 0.05);
              const opacity = diff === 0 ? 1 : Math.max(0.3, 1 - absDiff * 0.12);
              const zIndex = 100 - absDiff;
              const isApplying = applyingPath === img.path;
              const isPendingDelete = pendingDelete?.path === img.path;

              return (
                <div
                  key={img.path}
                  className={`carousel-card ${diff === 0 ? "active" : ""} ${isApplying ? "applying" : ""} ${isPendingDelete ? "pending-delete" : ""}`}
                  style={{
                    transform: `translateX(${xOffset}px) translateZ(${translateZ}px) rotateY(${rotateY}deg) scale(${scale})`,
                    zIndex,
                    opacity,
                  }}
                  onClick={() => handleSelect(img)}
                >
                  <img src={convertFileSrc(img.thumbnail)} alt="" loading="lazy" />
                  {diff === 0 ? (
                    <button
                      type="button"
                      className={`card-favorite ${img.favorite ? "on" : ""}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleFavorite(img);
                      }}
                      aria-pressed={img.favorite}
                      aria-label={img.favorite ? t.removeFavorite : t.addFavorite}
                      title={img.favorite ? t.removeFavorite : t.addFavorite}
                    >
                      <StarIcon filled={img.favorite} />
                    </button>
                  ) : (
                    img.favorite && (
                      <span className="card-favorite-mark" aria-hidden="true">
                        <StarIcon filled size={11} />
                      </span>
                    )
                  )}
                  {isApplying && (
                    <>
                      <div className="develop-overlay" />
                      <span className="develop-label">{t.developing}</span>
                    </>
                  )}
                </div>
              );
            })}
          </div>

          {pendingDelete ? (
            <div className="frame-caption frame-confirm" role="alertdialog" aria-label={t.confirmDeleteLabel}>
              <span className="frame-confirm-text">
                {t.confirmDeleteBefore} <strong>{pendingDelete.name}</strong> {t.confirmDeleteAfter}
              </span>
              <button type="button" className="frame-confirm-yes" onClick={confirmDelete}>
                {t.confirmMove} <kbd>Enter</kbd>
              </button>
              <button type="button" className="frame-confirm-no" onClick={() => setPendingDelete(null)}>
                {t.confirmCancel} <kbd>Esc</kbd>
              </button>
            </div>
          ) : (
            active && (
              <div className="frame-caption" aria-live="polite">
                <span className="frame-caption-number">
                  ▸ {String(activeIndex + 1).padStart(2, "0")}
                </span>
                <span className="frame-caption-total">/ {String(visible.length).padStart(2, "0")}</span>
                <span className="frame-caption-name">
                  {active.favorite && <StarIcon filled size={10} />}
                  {active.name}
                </span>
              </div>
            )
          )}

          {notice ? (
            <div className={`carousel-notice ${notice.tone}`} role="status">
              {notice.text}
            </div>
          ) : (
            <div className="carousel-hints" aria-hidden="true">
              <span><kbd>F</kbd> {t.hints.favorite}</span>
              <span><kbd>/</kbd> {t.hints.search}</span>
              <span><kbd>Del</kbd> {t.hints.delete}</span>
              <span><kbd>?</kbd> {t.hints.allShortcuts}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
