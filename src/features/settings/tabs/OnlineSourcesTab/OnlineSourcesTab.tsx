import { ReactNode, useState } from "react";
import { WallhavenSource } from "./wallhaven/WallhavenSource";
import { StockSource } from "./stock/StockSource";

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
  {
    id: "unsplash",
    label: "Unsplash",
    render: (onFeedback) => <StockSource key="unsplash" provider="unsplash" onFeedback={onFeedback} />,
  },
  {
    id: "pexels",
    label: "Pexels",
    render: (onFeedback) => <StockSource key="pexels" provider="pexels" onFeedback={onFeedback} />,
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
        <span className="content-eyebrow">Frame 03</span>
        <h2 className="content-header-title">Online Sources</h2>
        <p className="content-header-subtitle">
          Find wallpapers online and save them straight into your wallpaper folder. Wallhaven works right away; Unsplash and Pexels use your own free API key.
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
