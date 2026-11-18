import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import SettingsWindow from "./features/settings/SettingsWindow";
import { CarouselView } from "./features/carousel/CarouselView";
import { I18nProvider } from "./i18n/I18nProvider";
import "./styles/darkroom.css";
import "./App.css";

const windowLabel = getCurrentWebviewWindow().label;

export default function App() {
  return (
    <I18nProvider>
      {windowLabel === "settings" ? <SettingsWindow /> : <CarouselView />}
    </I18nProvider>
  );
}
