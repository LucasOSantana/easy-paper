import { useState, useEffect } from "react";
import { TauriService } from "../services/tauriCommands";
import { WallhavenPage, WallhavenSearch, WallhavenWallpaper } from "../types/wallhaven";

export type WallhavenFilters = Omit<WallhavenSearch, "page">;
export type DownloadState = "downloading" | "done";

type FeedbackHandler = (type: "success" | "error", message: string) => void;

export function useWallhavenSearch(onFeedback: FeedbackHandler) {
  const [filters, setFilters] = useState<WallhavenFilters>({
    query: "",
    sorting: "toplist",
    ratios: "",
    match_screen: true,
  });
  const [result, setResult] = useState<WallhavenPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloads, setDownloads] = useState<Record<string, DownloadState>>({});

  const search = async (page = 1, searchFilters = filters) => {
    try {
      setLoading(true);
      setResult(await TauriService.searchWallhaven({ ...searchFilters, page }));
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : "Wallhaven search failed.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    search();
  }, []);

  const changeFilters = (changes: Partial<WallhavenFilters>, searchNow: boolean) => {
    const updated = { ...filters, ...changes };
    setFilters(updated);
    if (searchNow) search(1, updated);
  };

  const download = async (wallpaper: WallhavenWallpaper) => {
    setDownloads((prev) => ({ ...prev, [wallpaper.id]: "downloading" }));
    try {
      await TauriService.downloadWallhaven(wallpaper.path);
      setDownloads((prev) => ({ ...prev, [wallpaper.id]: "done" }));
      onFeedback("success", "Wallpaper saved to your wallpaper folder.");
    } catch (err) {
      console.error(err);
      setDownloads(({ [wallpaper.id]: _, ...rest }) => rest);
      onFeedback("error", typeof err === "string" ? err : "Failed to download wallpaper.");
    }
  };

  return { filters, result, loading, downloads, search, changeFilters, download };
}
