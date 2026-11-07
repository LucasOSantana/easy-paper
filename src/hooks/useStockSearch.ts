import { useEffect, useState } from "react";
import { TauriService } from "../services/tauriCommands";
import { StockOrientation, StockPage, StockPhoto, StockProvider } from "../types/stock";
import { DownloadState } from "./useWallhavenSearch";

type FeedbackHandler = (type: "success" | "error", message: string) => void;

export interface StockFilters {
  query: string;
  orientation: StockOrientation;
}

/** Search + download state for one key-based provider. Searches only while `enabled`. */
export function useStockSearch(provider: StockProvider, enabled: boolean, onFeedback: FeedbackHandler) {
  const [filters, setFilters] = useState<StockFilters>({ query: "", orientation: "landscape" });
  const [result, setResult] = useState<StockPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [downloads, setDownloads] = useState<Record<string, DownloadState>>({});

  const search = async (page = 1, searchFilters = filters) => {
    try {
      setLoading(true);
      setResult(await TauriService.searchStock({ provider, ...searchFilters, page }));
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : "Search failed.");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (enabled) {
      search();
    } else {
      setResult(null);
    }
  }, [enabled, provider]);

  const changeFilters = (changes: Partial<StockFilters>, searchNow: boolean) => {
    const updated = { ...filters, ...changes };
    setFilters(updated);
    if (searchNow) search(1, updated);
  };

  const download = async (photo: StockPhoto) => {
    setDownloads((prev) => ({ ...prev, [photo.id]: "downloading" }));
    try {
      await TauriService.downloadStock(provider, photo);
      setDownloads((prev) => ({ ...prev, [photo.id]: "done" }));
      onFeedback("success", "Wallpaper saved to your wallpaper folder.");
    } catch (err) {
      console.error(err);
      setDownloads(({ [photo.id]: _, ...rest }) => rest);
      onFeedback("error", typeof err === "string" ? err : "Failed to download wallpaper.");
    }
  };

  return { filters, result, loading, downloads, search, changeFilters, download };
}
