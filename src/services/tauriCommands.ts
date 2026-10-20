import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../types/settings";
import { ImageItem } from "../types/wallpaper";

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
};
