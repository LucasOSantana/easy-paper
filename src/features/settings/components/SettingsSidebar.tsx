import { TabDefinition, TabId } from "../../../types/settings";

interface SettingsSidebarProps {
  tabs: TabDefinition[];
  activeTabId: TabId;
  onSelectTab: (tabId: TabId) => void;
}

export function SettingsSidebar({
  tabs,
  activeTabId,
  onSelectTab,
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
          strokeWidth="2.5"
        >
          <rect x="3" y="3" width="18" height="18" rx="3" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
        <div className="sidebar-brand-text">
          <span className="sidebar-brand-title">Easy Paper</span>
          <span className="sidebar-brand-sub">Settings</span>
        </div>
      </div>

      <div className="sidebar-category-header">Navigation</div>

      {tabs.map((tab) => {
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
            title={isDisabled ? "Feature coming soon" : tab.label}
          >
            <span className="tab-icon-wrap">{tab.icon(isActive)}</span>
            <span>{tab.label}</span>
            {tab.badge && <span className="tab-badge">{tab.badge}</span>}
          </button>
        );
      })}

    </aside>
  );
}
