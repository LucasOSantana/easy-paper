import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../services/tauriCommands";
import { UpdateInfo, UpdateProgress } from "../types/update";
import { useI18n } from "../i18n/I18nProvider";

type FeedbackHandler = (type: "success" | "error", message: string) => void;

/**
 * Shows the result of the check made when the app started (it may still be
 * running when this window opens, so we also listen for it), offers a manual
 * check and installs the update in place.
 */
export function useUpdateStatus(onFeedback: FeedbackHandler) {
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const { m } = useI18n();
  const t = m.settings.update;

  useEffect(() => {
    let cancelled = false;
    const stops: Array<() => void> = [];
    const keep = (stop: () => void) => (cancelled ? stop() : stops.push(stop));

    getVersion()
      .then((version) => !cancelled && setCurrentVersion(version))
      .catch(console.error);

    TauriService.getUpdateStatus()
      .then((status) => !cancelled && status && setInfo(status))
      .catch(console.error);

    listen<UpdateInfo>("update-status", (event) => setInfo(event.payload)).then(keep);
    listen<UpdateProgress>("update-progress", (event) => setProgress(event.payload)).then(keep);

    return () => {
      cancelled = true;
      stops.forEach((stop) => stop());
    };
  }, []);

  const checkNow = async () => {
    try {
      setChecking(true);
      const result = await TauriService.checkForUpdates();
      setInfo(result);
      onFeedback(
        "success",
        result.update_available ? t.newVersion(result.latest_version) : t.upToDate(result.current_version)
      );
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : t.checkFailed);
    } finally {
      setChecking(false);
    }
  };

  const installNow = async () => {
    try {
      setInstalling(true);
      setProgress(null);
      // Only returns on failure: on success the app closes and the installer reopens it.
      await TauriService.installUpdate();
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : t.installFailed);
    } finally {
      setInstalling(false);
      setProgress(null);
    }
  };

  return {
    currentVersion: info?.current_version ?? currentVersion,
    info,
    checking,
    checkNow,
    installing,
    progress,
    installNow,
  };
}
