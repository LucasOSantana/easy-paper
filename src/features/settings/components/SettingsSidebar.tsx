import { TabDefinition, TabId } from "../../../types/settings";
import { UpdateFooter } from "./UpdateFooter";

interface SettingsSidebarProps {
  tabs: TabDefinition[];
  activeTabId: TabId;
  onSelectTab: (tabId: TabId) => void;
  onFeedback: (type: "success" | "error", message: string) => void;
}

export function SettingsSidebar({
  tabs,
  activeTabId,
  onSelectTab,
  onFeedback,
}: SettingsSidebarProps) {
  return (
    <aside className="settings-sidebar">
      {/* Brand Header */}
      <div className="sidebar-brand">
        <svg
          className="sidebar-brand-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.5"
          aria-hidden="true"
        >
          {/* A single film frame with sprocket holes */}
          <rect x="3" y="2" width="18" height="20" rx="1" />
          <rect x="7.5" y="6.5" width="9" height="11" rx="0.5" />
          <path d="M5 5v1M5 9.5v1M5 14v1M5 18v1M19 5v1M19 9.5v1M19 14v1M19 18v1" />
        </svg>
        <div className="sidebar-brand-text">
          <span className="sidebar-brand-title">Easy Paper</span>
          <span className="sidebar-brand-sub">Settings</span>
        </div>
      </div>

      <nav className="sidebar-nav" aria-label="Settings sections">
        {tabs.map((tab, index) => {
          const isActive = tab.id === activeTabId;
          const isDisabled = Boolean(tab.disabled);

          return (
            <button
              key={tab.id}
              className={`sidebar-tab-button ${isActive ? "active" : ""} ${
                isDisabled ? "disabled" : ""
              }`}
              onClick={() => !isDisabled && onSelectTab(tab.id)}
              disabled={isDisabled}
              aria-current={isActive ? "page" : undefined}
              title={isDisabled ? "Feature coming soon" : tab.label}
            >
              <span className="tab-frame-number">{String(index + 1).padStart(2, "0")}</span>
              <span className="tab-label">{tab.label}</span>
              {tab.badge && <span className="tab-badge">{tab.badge}</span>}
            </button>
          );
        })}
      </nav>

      <UpdateFooter onFeedback={onFeedback} />
    </aside>
  );
}
