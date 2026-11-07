import { FolderSection } from "./FolderSection";
import { ShortcutSection } from "./ShortcutSection";
import { AutoRotateSection } from "./AutoRotateSection";
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
    changeAutoRotate,
    autoRotateUpdating,
    openFolderExplorer,
  } = settings;

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">Frame 01</span>
        <h2 className="content-header-title">Wallpapers & Shortcuts</h2>
        <p className="content-header-subtitle">
          Manage your local wallpaper folder, automatic rotation, and the global keyboard shortcut.
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

      <AutoRotateSection
        value={config.auto_rotate}
        disabled={loading || autoRotateUpdating}
        onChange={changeAutoRotate}
      />
      
    </div>
  );
}
