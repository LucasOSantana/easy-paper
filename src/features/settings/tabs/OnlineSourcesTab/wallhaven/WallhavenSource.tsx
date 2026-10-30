import { WallhavenSearchSection } from "./WallhavenSearchSection";
import { WallhavenResults } from "./WallhavenResults";
import { useWallhavenSearch } from "../../../../../hooks/useWallhavenSearch";

interface WallhavenSourceProps {
  onFeedback: (type: "success" | "error", message: string) => void;
}

export function WallhavenSource({ onFeedback }: WallhavenSourceProps) {
  const { filters, result, loading, downloads, search, changeFilters, download } =
    useWallhavenSearch(onFeedback);

  return (
    <>
      <WallhavenSearchSection
        filters={filters}
        loading={loading}
        onChange={changeFilters}
        onSearch={() => search()}
      />

      <WallhavenResults
        result={result}
        loading={loading}
        downloads={downloads}
        onDownload={download}
        onPageChange={search}
      />
    </>
  );
}
