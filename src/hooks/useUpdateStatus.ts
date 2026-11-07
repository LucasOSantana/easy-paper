import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { TauriService } from "../services/tauriCommands";
import { UpdateInfo } from "../types/update";

type FeedbackHandler = (type: "success" | "error", message: string) => void;

/**
 * Shows the result of the check made when the app started (it may still be
 * running when this window opens, so we also listen for it) and offers a
 * manual check.
 */
export function useUpdateStatus(onFeedback: FeedbackHandler) {
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    getVersion()
      .then((version) => !cancelled && setCurrentVersion(version))
      .catch(console.error);

    TauriService.getUpdateStatus()
      .then((status) => !cancelled && status && setInfo(status))
      .catch(console.error);

    listen<UpdateInfo>("update-status", (event) => setInfo(event.payload)).then((stop) => {
      if (cancelled) stop();
      else unlisten = stop;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const checkNow = async () => {
    try {
      setChecking(true);
      const result = await TauriService.checkForUpdates();
      setInfo(result);
      onFeedback(
        "success",
        result.update_available
          ? `Version ${result.latest_version} is available.`
          : `You're on the latest version (v${result.current_version}).`
      );
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : "Could not check for updates.");
    } finally {
      setChecking(false);
    }
  };

  return { currentVersion: info?.current_version ?? currentVersion, info, checking, checkNow };
}
