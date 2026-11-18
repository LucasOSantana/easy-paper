import { TabDefinition } from "../../../types/settings";
import { WallpapersTab } from "../tabs/WallpapersTab/WallpapersTab";
import { TaskbarTab } from "../tabs/TaskbarTab/TaskbarTab";
import { OnlineSourcesTab } from "../tabs/OnlineSourcesTab/OnlineSourcesTab";
import { BackupTab } from "../tabs/BackupTab/BackupTab";
import { AppearanceTab } from "../tabs/AppearanceTab/AppearanceTab";
import { Messages } from "../../../i18n/locales/en";
import { useAppSettings } from "../../../hooks/useAppSettings";

export function createTabsRegistry(
  settings: ReturnType<typeof useAppSettings>,
  m: Messages
): TabDefinition[] {
  return [
    {
      id: "wallpapers",
      label: m.settings.tabs.wallpapers,
      render: () => <WallpapersTab settings={settings} />,
    },
    {
      id: "taskbar",
      label: m.settings.tabs.taskbar,
      render: () => <TaskbarTab settings={settings} />,
    },
    {
      id: "online_sources",
      label: m.settings.tabs.onlineSources,
      render: () => <OnlineSourcesTab onFeedback={settings.showFeedback} />,
    },
    {
      id: "backup",
      label: m.settings.tabs.backup,
      render: () => <BackupTab settings={settings} />,
    },
    {
      id: "appearance",
      label: m.settings.tabs.appearance,
      render: () => <AppearanceTab settings={settings} />,
    },
  ];
}
