import { useState, useEffect, useCallback } from "react";
import { AppConfig, AutoRotateConfig, FeedbackState } from "../types/settings";
import { TauriService } from "../services/tauriCommands";

export function useAppSettings() {
  const [config, setConfig] = useState<AppConfig>({
    shortcut: "super+j",
    images_path: "",
    taskbar_transparent: false,
    auto_rotate: {
      enabled: false,
      trigger: "interval",
      interval_minutes: 180,
      order: "shuffle",
    },
  });
  const [loading, setLoading] = useState(true);
  const [taskbarUpdating, setTaskbarUpdating] = useState(false);
  const [autoRotateUpdating, setAutoRotateUpdating] = useState(false);
  const [feedback, setFeedback] = useState<FeedbackState | null>(null);

  const showFeedback = useCallback((type: "success" | "error", message: string) => {
    setFeedback({ type, message });
    const timer = setTimeout(() => {
      setFeedback(null);
    }, 4500);
    return () => clearTimeout(timer);
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      setLoading(true);
      const data = await TauriService.fetchConfig();
      setConfig(data);
    } catch (err) {
      console.error(err);
      showFeedback("error", "Failed to load application configuration.");
    } finally {
      setLoading(false);
    }
  }, [showFeedback]);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  const changeFolder = async () => {
    try {
      const selected = await TauriService.pickFolder();
      if (!selected || selected.trim() === "") return;

      const updated = await TauriService.updateConfig(config.shortcut, selected.trim());
      setConfig(updated);
      showFeedback("success", "Wallpaper folder updated successfully.");
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : "Failed to select folder.");
    }
  };

  const changeShortcut = async (newShortcut: string) => {
    try {
      const updated = await TauriService.updateConfig(newShortcut, config.images_path);
      setConfig(updated);
      showFeedback("success", `Shortcut updated to "${newShortcut}".`);
      return true;
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : "Unable to register the specified shortcut.");
      return false;
    }
  };

  const changeTaskbarTransparent = async (transparent: boolean) => {
    try {
      setTaskbarUpdating(true);
      const updated = await TauriService.setTaskbarTransparent(transparent);
      setConfig(updated);
      showFeedback(
        "success",
        transparent ? "Taskbar is now transparent." : "Taskbar restored to the Windows default."
      );
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : "Unable to update the taskbar.");
    } finally {
      setTaskbarUpdating(false);
    }
  };

  const changeAutoRotate = async (settings: AutoRotateConfig) => {
    const wasEnabled = config.auto_rotate.enabled;
    try {
      setAutoRotateUpdating(true);
      const updated = await TauriService.setAutoRotate(settings);
      setConfig(updated);
      if (wasEnabled !== settings.enabled) {
        showFeedback(
          "success",
          settings.enabled ? "Automatic rotation turned on." : "Automatic rotation turned off."
        );
      }
      return true;
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : "Unable to update automatic rotation.");
      return false;
    } finally {
      setAutoRotateUpdating(false);
    }
  };

  const openFolderExplorer = async () => {
    if (!config.images_path) return;
    try {
      await TauriService.openFolder(config.images_path);
    } catch (err) {
      console.error(err);
    }
  };

  const dismissSettings = async () => {
    try {
      await TauriService.closeSettings();
    } catch (err) {
      console.error(err);
    }
  };

  return {
    config,
    loading,
    taskbarUpdating,
    autoRotateUpdating,
    feedback,
    showFeedback,
    changeFolder,
    changeShortcut,
    changeTaskbarTransparent,
    changeAutoRotate,
    openFolderExplorer,
    dismissSettings,
  };
}
