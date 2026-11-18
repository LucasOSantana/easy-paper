export type StockProvider = "unsplash" | "pexels";
export type StockOrientation = "landscape" | "portrait" | "";

export interface StockSearch {
  provider: StockProvider;
  query: string;
  orientation: StockOrientation;
  page: number;
}

export interface StockPhoto {
  id: string;
  width: number;
  height: number;
  thumb_url: string;
  download_url: string;
  page_url: string;
  author_name: string;
  author_url: string;
  download_location: string | null;
}

export interface StockPage {
  photos: StockPhoto[];
  page: number;
  has_next: boolean;
}

export interface StockProviderInfo {
  id: StockProvider;
  name: string;
  /** Where the user creates their free key. */
  keyUrl: string;
  /** Provider home, linked from every results page (attribution). */
  homeUrl: string;
}

export const STOCK_PROVIDERS: Record<StockProvider, StockProviderInfo> = {
  unsplash: {
    id: "unsplash",
    name: "Unsplash",
    keyUrl: "https://unsplash.com/oauth/applications",
    homeUrl: "https://unsplash.com/?utm_source=easy_paper&utm_medium=referral",
  },
  pexels: {
    id: "pexels",
    name: "Pexels",
    keyUrl: "https://www.pexels.com/api/",
    homeUrl: "https://www.pexels.com",
  },
};
