import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import SettingsWindow from "./features/settings/SettingsWindow";
import { CarouselView } from "./features/carousel/CarouselView";
import "./App.css";

export default function App() {
  const [windowLabel, setWindowLabel] = useState<string>("main");

  useEffect(() => {
    try {
      const win = getCurrentWebviewWindow();
      setWindowLabel(win.label);
    } catch (err) {
      console.warn("Could not determine window label:", err);
    }
  }, []);

  if (windowLabel === "settings") {
    return <SettingsWindow />;
  }

  return <CarouselView />;
}
