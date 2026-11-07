import { useState, useEffect } from "react";
import { TabId } from "../../types/settings";
import { useAppSettings } from "../../hooks/useAppSettings";
import { createTabsRegistry } from "./config/tabsRegistry";
import { SettingsSidebar } from "./components/SettingsSidebar";
import { FeedbackToast } from "./components/FeedbackToast";
import "./settings.css";

export default function SettingsWindow() {
  const [activeTabId, setActiveTabId] = useState<TabId>("wallpapers");
  const settings = useAppSettings();
  const tabs = createTabsRegistry(settings);

  // Close window on Escape key
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        settings.dismissSettings();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [settings]);

  const activeTab = tabs.find((t) => t.id === activeTabId) || tabs[0];

  return (
    <div className="settings-root">
      <div className="settings-workspace">
        <SettingsSidebar
          tabs={tabs}
          activeTabId={activeTabId}
          onSelectTab={setActiveTabId}
          onFeedback={settings.showFeedback}
        />

        <main className="settings-content-viewport">
          {activeTab.render()}
          <FeedbackToast feedback={settings.feedback} />
        </main>
      </div>
    </div>
  );
}
