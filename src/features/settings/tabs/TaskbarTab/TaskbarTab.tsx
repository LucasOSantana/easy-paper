import { TransparencySection } from "./TransparencySection";
import { useAppSettings } from "../../../../hooks/useAppSettings";

interface TaskbarTabProps {
  settings: ReturnType<typeof useAppSettings>;
}

export function TaskbarTab({ settings }: TaskbarTabProps) {
  const { config, loading, taskbarUpdating, changeTaskbarTransparent } = settings;

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">Frame 02</span>
        <h2 className="content-header-title">Taskbar</h2>
        <p className="content-header-subtitle">
          Customize how the Windows taskbar looks. Changes apply instantly, are restored every time Easy Paper starts, and are undone when it quits.
        </p>
      </div>

      <TransparencySection
        transparent={config.taskbar_transparent}
        disabled={loading || taskbarUpdating}
        onChange={changeTaskbarTransparent}
      />
    </div>
  );
}
