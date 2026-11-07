import { useEffect, useState } from "react";
import { TauriService } from "../../../../../services/tauriCommands";
import { useStockSearch } from "../../../../../hooks/useStockSearch";
import { STOCK_PROVIDERS, StockProvider } from "../../../../../types/stock";
import { ApiKeySetup } from "./ApiKeySetup";
import { StockSearchSection } from "./StockSearchSection";
import { StockResults } from "./StockResults";

interface StockSourceProps {
  provider: StockProvider;
  onFeedback: (type: "success" | "error", message: string) => void;
}

/** A key-based photo source: asks for the user's key, then searches. */
export function StockSource({ provider, onFeedback }: StockSourceProps) {
  const info = STOCK_PROVIDERS[provider];
  const [hasKey, setHasKey] = useState<boolean | null>(null);
  const [replacingKey, setReplacingKey] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setHasKey(null);
    setReplacingKey(false);
    TauriService.hasStockApiKey(provider)
      .then((value) => !cancelled && setHasKey(value))
      .catch(() => !cancelled && setHasKey(false));
    return () => {
      cancelled = true;
    };
  }, [provider]);

  const ready = hasKey === true && !replacingKey;
  const { filters, result, loading, downloads, search, changeFilters, download } = useStockSearch(
    provider,
    ready,
    onFeedback
  );

  const removeKey = async () => {
    try {
      await TauriService.removeStockApiKey(provider);
      setHasKey(false);
      onFeedback("success", `${info.name} key removed from this PC.`);
    } catch (err) {
      console.error(err);
      onFeedback("error", typeof err === "string" ? err : "Could not remove the key.");
    }
  };

  if (hasKey === null) {
    return <p className="results-empty">Loading…</p>;
  }

  if (!ready) {
    return (
      <ApiKeySetup
        info={info}
        onFeedback={onFeedback}
        onSaved={() => {
          setHasKey(true);
          setReplacingKey(false);
        }}
        onCancel={replacingKey ? () => setReplacingKey(false) : undefined}
      />
    );
  }

  return (
    <>
      <StockSearchSection
        info={info}
        filters={filters}
        loading={loading}
        onChange={changeFilters}
        onSearch={() => search()}
      />

      <StockResults
        info={info}
        result={result}
        loading={loading}
        downloads={downloads}
        onDownload={download}
        onPageChange={search}
      />

      <div className="key-footer">
        <span>Using your {info.name} key</span>
        <button className="link-button" type="button" onClick={() => setReplacingKey(true)}>
          Change key
        </button>
        <button className="link-button" type="button" onClick={removeKey}>
          Remove key
        </button>
      </div>
    </>
  );
}
