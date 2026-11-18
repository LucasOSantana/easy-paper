import { ReactNode, useState } from "react";
import { WallhavenSource } from "./wallhaven/WallhavenSource";
import { StockSource } from "./stock/StockSource";
import { useI18n } from "../../../../i18n/I18nProvider";

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
  const { m } = useI18n();

  return (
    <div className="tab-pane">
      <div className="content-header-block">
        <span className="content-eyebrow">{m.settings.frame(3)}</span>
        <h2 className="content-header-title">{m.settings.online.title}</h2>
        <p className="content-header-subtitle">{m.settings.online.subtitle}</p>
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
