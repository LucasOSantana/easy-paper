import { useState, useEffect, useCallback, useRef } from "react";
import { AppConfig, AutoRotateConfig, FeedbackState } from "../types/settings";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../services/tauriCommands";
import { CONFIG_CHANGED_EVENT } from "../types/sync";
import { LanguageSetting, useI18n } from "../i18n/I18nProvider";

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
      favorites_only: false,
    },
    favorites: [],
    sync: { folder: "" },
    settings_updated_at: 0,
    language: "system",
  });
  const [loading, setLoading] = useState(true);
  const [taskbarUpdating, setTaskbarUpdating] = useState(false);
  const [autoRotateUpdating, setAutoRotateUpdating] = useState(false);
  const [feedback, setFeedback] = useState<FeedbackState | null>(null);
  const { m } = useI18n();
  // Read through a ref so callbacks keep a stable identity across language changes.
  const messages = useRef(m);
  messages.current = m;
  const fb = () => messages.current.settings.feedback;

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
      showFeedback("error", fb().loadConfigFailed);
    } finally {
      setLoading(false);
    }
  }, [showFeedback]);

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  // Stay current with changes made from the tray, a restored backup or sync.
  useEffect(() => {
    const unlisten = listen<AppConfig>(CONFIG_CHANGED_EVENT, (event) => setConfig(event.payload));
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  const changeFolder = async () => {
    try {
      const selected = await TauriService.pickFolder();
      if (!selected || selected.trim() === "") return;

      const updated = await TauriService.updateConfig(config.shortcut, selected.trim());
      setConfig(updated);
      showFeedback("success", fb().folderUpdated);
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : fb().folderFailed);
    }
  };

  const changeShortcut = async (newShortcut: string) => {
    try {
      const updated = await TauriService.updateConfig(newShortcut, config.images_path);
      setConfig(updated);
      showFeedback("success", fb().shortcutUpdated(newShortcut));
      return true;
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : fb().shortcutFailed);
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
        transparent ? fb().taskbarOn : fb().taskbarOff
      );
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : fb().taskbarFailed);
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
          settings.enabled ? fb().rotationOn : fb().rotationOff
        );
      }
      return true;
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : fb().rotationFailed);
      return false;
    } finally {
      setAutoRotateUpdating(false);
    }
  };

  const changeLanguage = async (language: LanguageSetting) => {
    try {
      setConfig(await TauriService.setLanguage(language));
      return true;
    } catch (err) {
      console.error(err);
      showFeedback("error", typeof err === "string" ? err : fb().languageFailed);
      return false;
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
    changeLanguage,
    openFolderExplorer,
    dismissSettings,
  };
}
