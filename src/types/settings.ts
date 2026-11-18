import { ReactNode } from "react";
import type { LanguageSetting } from "../i18n/I18nProvider";

export type RotationTrigger = "on_startup" | "interval";
export type RotationOrder = "shuffle" | "random";

export interface AutoRotateConfig {
  enabled: boolean;
  trigger: RotationTrigger;
  interval_minutes: number;
  order: RotationOrder;
  /** Only rotate through favorites (falls back to the whole folder when there are none). */
  favorites_only: boolean;
}

export const MIN_ROTATION_INTERVAL_MINUTES = 5;
export const MAX_ROTATION_INTERVAL_MINUTES = 7 * 24 * 60;

export interface AppConfig {
  shortcut: string;
  images_path: string;
  taskbar_transparent: boolean;
  auto_rotate: AutoRotateConfig;
  /** File names inside `images_path` marked as favorite. */
  favorites: string[];
  sync: SyncConfig;
  settings_updated_at: number;
  /** "system" follows the Windows display language. */
  language: LanguageSetting;
}

export interface SyncConfig {
  /** Sync folder; empty when sync is off. */
  folder: string;
}

export type TabId = "wallpapers" | "taskbar" | "online_sources" | "backup" | "appearance" | "about";

export interface TabDefinition {
  id: TabId;
  label: string;
  disabled?: boolean;
  badge?: string;
  render: () => ReactNode;
}

export interface FeedbackState {
  type: "success" | "error";
  message: string;
}
