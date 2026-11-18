import { invoke } from "@tauri-apps/api/core";
import { AppConfig, AutoRotateConfig } from "../types/settings";
import { ImageItem } from "../types/wallpaper";
import { WallhavenPage, WallhavenSearch } from "../types/wallhaven";
import { StockPage, StockPhoto, StockProvider, StockSearch } from "../types/stock";
import { UpdateInfo } from "../types/update";
import { BackupSummary, SyncFolderInfo, SyncStatus } from "../types/sync";
import type { Language, LanguageSetting } from "../i18n/I18nProvider";

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

  async setFavorite(path: string, favorite: boolean): Promise<boolean> {
    return invoke<boolean>("set_favorite", { path, favorite });
  },

  /** Moves the wallpaper to the Recycle Bin. */
  async deleteWallpaper(path: string): Promise<void> {
    return invoke("delete_wallpaper", { path });
  },

  /** Opens Explorer with the wallpaper selected. */
  async revealWallpaper(path: string): Promise<void> {
    return invoke("reveal_wallpaper", { path });
  },

  /** Asks for a destination folder; resolves `null` when cancelled. */
  async exportBackup(includeImages: boolean): Promise<BackupSummary | null> {
    return invoke<BackupSummary | null>("export_backup", { includeImages });
  },

  /** Asks for a backup file; resolves `null` when cancelled. */
  async importBackup(): Promise<BackupSummary | null> {
    return invoke<BackupSummary | null>("import_backup");
  },

  async pickSyncFolder(): Promise<string | null> {
    return invoke<string | null>("pick_sync_folder");
  },

  async inspectSyncFolder(folder: string): Promise<SyncFolderInfo | null> {
    return invoke<SyncFolderInfo | null>("inspect_sync_folder", { folder });
  },

  async connectSyncFolder(folder: string, useFolderSettings: boolean): Promise<SyncStatus> {
    return invoke<SyncStatus>("connect_sync_folder", { folder, useFolderSettings });
  },

  async disconnectSync(): Promise<AppConfig> {
    return invoke<AppConfig>("disconnect_sync");
  },

  async syncNow(): Promise<SyncStatus> {
    return invoke<SyncStatus>("sync_now");
  },

  async getSyncStatus(): Promise<SyncStatus> {
    return invoke<SyncStatus>("get_sync_status");
  },

  /** Language the UI must use, with "system" resolved by the backend. */
  async getLanguage(): Promise<Language> {
    return invoke<Language>("get_language");
  },

  async setLanguage(language: LanguageSetting): Promise<AppConfig> {
    return invoke<AppConfig>("set_language", { language });
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

  /** Downloads and runs the installer. On success the app closes, so this only resolves on failure. */
  async installUpdate(): Promise<void> {
    return invoke("install_update");
  },
};
