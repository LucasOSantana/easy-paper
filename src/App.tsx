import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import SettingsWindow from "./features/settings/SettingsWindow";
import { CarouselView } from "./features/carousel/CarouselView";
import "./App.css";

const windowLabel = getCurrentWebviewWindow().label;

export default function App() {
  return windowLabel === "settings" ? <SettingsWindow /> : <CarouselView />;
}
