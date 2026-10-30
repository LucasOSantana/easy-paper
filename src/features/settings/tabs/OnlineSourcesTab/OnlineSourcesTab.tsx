import { ReactNode, useState } from "react";
import { WallhavenSource } from "./wallhaven/WallhavenSource";

type FeedbackHandler = (type: "success" | "error", message: string) => void;

interface OnlineSource {
  id: string;
  label: string;
  render: (onFeedback: FeedbackHandler) => ReactNode;
}

const SOURCES: OnlineSource[] = [
  {
    id: "wallhaven",
    label: "Wallhaven",
    render: (onFeedback) => <WallhavenSource onFeedback={onFeedback} />,
  },
];

interface OnlineSourcesTabProps {
  onFeedback: FeedbackHandler;
}

export function OnlineSourcesTab({ onFeedback }: OnlineSourcesTabProps) {
  const [activeSourceId, setActiveSourceId] = useState(SOURCES[0].id);
  const activeSource = SOURCES.find((source) => source.id === activeSourceId) ?? SOURCES[0];

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <h2 className="content-header-title">Online Sources</h2>
        <p className="content-header-subtitle">
          Find wallpapers online and save them straight into your wallpaper folder.
        </p>
      </div>

      <div className="source-tabs" role="tablist">
        {SOURCES.map((source) => (
          <button
            key={source.id}
            type="button"
            role="tab"
            aria-selected={source.id === activeSource.id}
            className={`source-tab ${source.id === activeSource.id ? "active" : ""}`}
            onClick={() => setActiveSourceId(source.id)}
          >
            {source.label}
          </button>
        ))}
      </div>

      {activeSource.render(onFeedback)}
    </div>
  );
}
