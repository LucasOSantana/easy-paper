import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface ImageItem {
  name: string;
  path: string;
  thumbnail: string;
}

export default function App() {
  const [images, setImages] = useState<ImageItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeIndex, setActiveIndex] = useState(0);
  const [applyingPath, setApplyingPath] = useState<string | null>(null);
  const isScrollingRef = useRef(false);

  async function loadImages() {
    try {
      const list = await invoke<ImageItem[]>("get_images");
      setImages(list);
      if (list.length > 0) {
        setActiveIndex((prev) => Math.min(prev, list.length - 1));
      }
    } catch (error) {
      console.error(error);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadImages();

    const handleFocus = () => {
      loadImages();
    };

    window.addEventListener("focus", handleFocus);
    return () => {
      window.removeEventListener("focus", handleFocus);
    };
  }, []);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (images.length === 0 || applyingPath) return;
      if (e.key === "ArrowRight") {
        setActiveIndex((prev) => Math.min(prev + 1, images.length - 1));
      } else if (e.key === "ArrowLeft") {
        setActiveIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter" || e.key === " ") {
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
      await invoke("set_wallpaper", { path: img.path });
    } catch (error) {
      console.error(error);
    }

    setTimeout(async () => {
      try {
        await invoke("hide_window");
      } catch (e) {
        console.error(e);
      } finally {
        setApplyingPath(null);
      }
    }, 650);
  };

  return (
    <div className="carousel-container" onWheel={handleWheel}>
      {loading && images.length === 0 ? (
        <div className="carousel-empty">Carregando imagens...</div>
      ) : images.length === 0 ? (
        <div className="carousel-empty">
          <p>Nenhuma imagem encontrada na pasta configurada.</p>
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
                  <img src={img.thumbnail} alt="" loading="lazy" />
                  {isApplying && (
                    <div className="applied-overlay">
                      <div className="ripple-wave" />
                      <div className="applied-check">✓</div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
