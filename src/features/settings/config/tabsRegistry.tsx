import { TabDefinition } from "../../../types/settings";
import { WallpapersTab } from "../tabs/WallpapersTab/WallpapersTab";
import { TaskbarTab } from "../tabs/TaskbarTab/TaskbarTab";
import { OnlineSourcesTab } from "../tabs/OnlineSourcesTab/OnlineSourcesTab";
import { useAppSettings } from "../../../hooks/useAppSettings";

export function createTabsRegistry(
  settings: ReturnType<typeof useAppSettings>
): TabDefinition[] {
  return [
    {
      id: "wallpapers",
      label: "Wallpapers",
      render: () => <WallpapersTab settings={settings} />,
    },
    {
      id: "taskbar",
      label: "Taskbar",
      render: () => <TaskbarTab settings={settings} />,
    },
    {
      id: "online_sources",
      label: "Online Sources",
      render: () => <OnlineSourcesTab onFeedback={settings.showFeedback} />,
    },
    {
      id: "appearance",
      label: "Appearance",
      badge: "Soon",
      disabled: true,
      render: () => null,
    },
  ];
}
