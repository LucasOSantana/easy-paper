import { ReactNode } from "react";

export type RotationTrigger = "on_startup" | "interval";
export type RotationOrder = "shuffle" | "random";

export interface AutoRotateConfig {
  enabled: boolean;
  trigger: RotationTrigger;
  interval_minutes: number;
  order: RotationOrder;
}

export const MIN_ROTATION_INTERVAL_MINUTES = 5;
export const MAX_ROTATION_INTERVAL_MINUTES = 7 * 24 * 60;

export interface AppConfig {
  shortcut: string;
  images_path: string;
  taskbar_transparent: boolean;
  auto_rotate: AutoRotateConfig;
}

export type TabId = "wallpapers" | "taskbar" | "online_sources" | "appearance" | "about";

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
