import { TabDefinition } from "../../../types/settings";
import { WallpapersTab } from "../tabs/WallpapersTab/WallpapersTab";
import { TaskbarTab } from "../tabs/TaskbarTab/TaskbarTab";
import { useAppSettings } from "../../../hooks/useAppSettings";

export function createTabsRegistry(
  settings: ReturnType<typeof useAppSettings>
): TabDefinition[] {
  return [
    {
      id: "wallpapers",
      label: "Wallpapers",
      icon: () => (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
      ),
      component: () => <WallpapersTab settings={settings} />,
    },
    {
      id: "taskbar",
      label: "Taskbar",
      icon: () => (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <rect x="2" y="3" width="20" height="18" rx="2" />
          <line x1="2" y1="16" x2="22" y2="16" />
          <line x1="6" y1="18.5" x2="8" y2="18.5" />
          <line x1="11" y1="18.5" x2="13" y2="18.5" />
        </svg>
      ),
      component: () => <TaskbarTab settings={settings} />,
    },
    {
      id: "online_sources",
      label: "Online Sources",
      badge: "Soon",
      disabled: true,
      icon: () => (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
      ),
      component: () => <div />,
    },
    {
      id: "appearance",
      label: "Appearance",
      badge: "Soon",
      disabled: true,
      icon: () => (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      ),
      component: () => <div />,
    },
  ];
}
