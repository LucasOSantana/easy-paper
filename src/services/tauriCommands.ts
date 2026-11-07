import { invoke } from "@tauri-apps/api/core";
import { AppConfig, AutoRotateConfig } from "../types/settings";
import { ImageItem } from "../types/wallpaper";
import { WallhavenPage, WallhavenSearch } from "../types/wallhaven";
import { StockPage, StockPhoto, StockProvider, StockSearch } from "../types/stock";
import { UpdateInfo } from "../types/update";

/**
 * Service to communicate with Tauri Rust backend commands.
 * Decouples React UI components from low-level IPC invocations.
 */
export const TauriService = {
  async fetchConfig(): Promise<AppConfig> {
    return invoke<AppConfig>("get_config");
  },

  async updateConfig(shortcut: string, imagesPath: string): Promise<AppConfig> {
    return invoke<AppConfig>("update_config", {
      shortcut,
      imagesPath,
    });
  },

  async setTaskbarTransparent(transparent: boolean): Promise<AppConfig> {
    return invoke<AppConfig>("set_taskbar_transparent", { transparent });
  },

  async setAutoRotate(settings: AutoRotateConfig): Promise<AppConfig> {
    return invoke<AppConfig>("set_auto_rotate", { settings });
  },

  async pickFolder(): Promise<string | null> {
    return invoke<string | null>("pick_folder");
  },

  async openFolder(path: string): Promise<void> {
    return invoke("open_folder_path", { path });
  },

  async openSettings(): Promise<void> {
    return invoke("open_settings_window");
  },

  async closeSettings(): Promise<void> {
    return invoke("close_settings_window");
  },

  async getImages(): Promise<ImageItem[]> {
    return invoke<ImageItem[]>("get_images");
  },

  async setWallpaper(path: string): Promise<void> {
    return invoke("set_wallpaper", { path });
  },

  async hideMainWindow(): Promise<void> {
    return invoke("hide_window");
  },

  async searchWallhaven(search: WallhavenSearch): Promise<WallhavenPage> {
    return invoke<WallhavenPage>("search_wallhaven", { search });
  },

  async downloadWallhaven(url: string): Promise<void> {
    return invoke("download_wallhaven", { url });
  },

  async hasStockApiKey(provider: StockProvider): Promise<boolean> {
    return invoke<boolean>("has_stock_api_key", { provider });
  },

  async saveStockApiKey(provider: StockProvider, key: string): Promise<void> {
    return invoke("save_stock_api_key", { provider, key });
  },

  async removeStockApiKey(provider: StockProvider): Promise<void> {
    return invoke("remove_stock_api_key", { provider });
  },

  async searchStock(search: StockSearch): Promise<StockPage> {
    return invoke<StockPage>("search_stock", { search });
  },

  async downloadStock(provider: StockProvider, photo: StockPhoto): Promise<void> {
    return invoke("download_stock", { provider, photo });
  },

  async getUpdateStatus(): Promise<UpdateInfo | null> {
    return invoke<UpdateInfo | null>("get_update_status");
  },

  async checkForUpdates(): Promise<UpdateInfo> {
    return invoke<UpdateInfo>("check_for_updates");
  },
};
