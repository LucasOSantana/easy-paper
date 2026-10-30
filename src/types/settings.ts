import { ReactNode } from "react";

export interface AppConfig {
  shortcut: string;
  images_path: string;
  taskbar_transparent: boolean;
}

export type TabId = "wallpapers" | "taskbar" | "online_sources" | "appearance" | "about";

export interface TabDefinition {
  id: TabId;
  label: string;
  icon: (active: boolean) => ReactNode;
  disabled?: boolean;
  badge?: string;
  component: React.ComponentType;
}

export interface FeedbackState {
  type: "success" | "error";
  message: string;
}
