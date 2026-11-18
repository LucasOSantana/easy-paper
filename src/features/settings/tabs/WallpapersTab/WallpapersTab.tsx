import { FolderSection } from "./FolderSection";
import { ShortcutSection } from "./ShortcutSection";
import { AutoRotateSection } from "./AutoRotateSection";
import { useAppSettings } from "../../../../hooks/useAppSettings";
import { useI18n } from "../../../../i18n/I18nProvider";

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
  const { m } = useI18n();
  const t = m.settings.wallpapers;

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">{m.settings.frame(1)}</span>
        <h2 className="content-header-title">{t.title}</h2>
        <p className="content-header-subtitle">{t.subtitle}</p>
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
        favoritesCount={config.favorites.length}
        disabled={loading || autoRotateUpdating}
        onChange={changeAutoRotate}
      />
      
    </div>
  );
}
