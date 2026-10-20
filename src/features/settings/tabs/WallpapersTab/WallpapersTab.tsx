import { FolderSection } from "./FolderSection";
import { ShortcutSection } from "./ShortcutSection";
import { useAppSettings } from "../../../../hooks/useAppSettings";

interface WallpapersTabProps {
  settings: ReturnType<typeof useAppSettings>;
}

export function WallpapersTab({ settings }: WallpapersTabProps) {
  const {
    config,
    loading,
    changeFolder,
    changeShortcut,
    openFolderExplorer,
  } = settings;

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <h2 className="content-header-title">Wallpapers & Shortcuts</h2>
        <p className="content-header-subtitle">
          Manage your local wallpaper image repository and custom global keyboard shortcut.
        </p>
      </div>

      <FolderSection
        imagesPath={config.images_path}
        loading={loading}
        onPickFolder={changeFolder}
        onOpenFolder={openFolderExplorer}
      />

      <ShortcutSection
        currentShortcut={config.shortcut}
        onUpdateShortcut={changeShortcut}
      />
    </div>
  );
}
