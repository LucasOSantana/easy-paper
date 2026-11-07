import { useEffect, useState, useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ImageItem } from "../../types/wallpaper";
import { TauriService } from "../../services/tauriCommands";

export function CarouselView() {
  const [images, setImages] = useState<ImageItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeIndex, setActiveIndex] = useState(0);
  const [applyingPath, setApplyingPath] = useState<string | null>(null);
  const isScrollingRef = useRef(false);

  async function loadImages() {
    try {
      const list = await TauriService.getImages();
      setImages(list);
      if (list.length > 0) {
        setActiveIndex((prev) => Math.min(prev, list.length - 1));
      }
    } catch (error) {
      console.error("Failed to load wallpapers:", error);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadImages();

    const handleFocus = () => {
      (document.activeElement as HTMLElement | null)?.blur();
      loadImages();
    };

    window.addEventListener("focus", handleFocus);
    return () => {
      window.removeEventListener("focus", handleFocus);
    };
  }, []);

  const handleOpenSettings = async () => {
    try {
      await TauriService.openSettings();
    } catch (err) {
      console.error("Failed to open settings window:", err);
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "," && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        handleOpenSettings();
        return;
      }

      if (images.length === 0 || applyingPath) return;
      if (e.key === "ArrowRight") {
        setActiveIndex((prev) => Math.min(prev + 1, images.length - 1));
      } else if (e.key === "ArrowLeft") {
        setActiveIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        if (images[activeIndex]) {
          handleSelect(images[activeIndex], activeIndex);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [images, activeIndex, applyingPath]);

  const handleWheel = (e: React.WheelEvent) => {
    if (images.length === 0 || isScrollingRef.current || applyingPath) return;
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(delta) < 20) return;

    isScrollingRef.current = true;
    setTimeout(() => {
      isScrollingRef.current = false;
    }, 150);

    if (delta > 0) {
      setActiveIndex((prev) => Math.min(prev + 1, images.length - 1));
    } else {
      setActiveIndex((prev) => Math.max(prev - 1, 0));
    }
  };

  const handleSelect = async (img: ImageItem, index: number) => {
    if (applyingPath) return;

    setActiveIndex(index);
    setApplyingPath(img.path);

    try {
      await TauriService.setWallpaper(img.path);
    } catch (error) {
      console.error("Failed to apply desktop wallpaper:", error);
    }

    setTimeout(async () => {
      try {
        await TauriService.hideMainWindow();
      } catch (err) {
        console.error("Failed to hide window:", err);
      } finally {
        setApplyingPath(null);
      }
    }, 650);
  };

  return (
    <div className="carousel-container" onWheel={handleWheel}>
      {/* Quick Settings Action Bar */}
      <div className="carousel-topbar">
        <button
          className="btn-settings-trigger"
          onClick={handleOpenSettings}
          title="Open Settings (Ctrl + ,)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
          <span>Settings</span>
          <kbd className="btn-settings-hint">Ctrl ,</kbd>
        </button>
      </div>

      {loading && images.length === 0 ? (
        <div className="carousel-empty">
          <span className="carousel-empty-eyebrow">Loading roll…</span>
        </div>
      ) : images.length === 0 ? (
        <div className="carousel-empty">
          <span className="carousel-empty-eyebrow">Empty roll</span>
          <p>No wallpapers found in the configured directory.</p>
          <button className="btn-settings-trigger" onClick={handleOpenSettings}>
            Configure Wallpaper Folder
          </button>
        </div>
      ) : (
        <div className="carousel-stage">
          <div className="carousel-track">
            {images.map((img, index) => {
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

              return (
                <div
                  key={img.path}
                  className={`carousel-card ${diff === 0 ? "active" : ""} ${isApplying ? "applying" : ""}`}
                  style={{
                    transform: `translateX(${xOffset}px) translateZ(${translateZ}px) rotateY(${rotateY}deg) scale(${scale})`,
                    zIndex,
                    opacity,
                  }}
                  onClick={() => handleSelect(img, index)}
                >
                  <img src={convertFileSrc(img.thumbnail)} alt="" loading="lazy" />
                  {isApplying && (
                    <>
                      <div className="develop-overlay" />
                      <span className="develop-label">Developing</span>
                    </>
                  )}
                </div>
              );
            })}
          </div>

          {images[activeIndex] && (
            <div className="frame-caption" aria-live="polite">
              <span className="frame-caption-number">
                ▸ {String(activeIndex + 1).padStart(2, "0")}
              </span>
              <span className="frame-caption-total">/ {String(images.length).padStart(2, "0")}</span>
              <span className="frame-caption-name">{images[activeIndex].name}</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
