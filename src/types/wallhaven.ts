export type WallhavenSorting = "relevance" | "date_added" | "toplist" | "views" | "favorites" | "random";

export interface WallhavenSearch {
  query: string;
  sorting: WallhavenSorting;
  ratios: string;
  match_screen: boolean;
  page: number;
}

export interface WallhavenWallpaper {
  id: string;
  path: string;
  resolution: string;
  thumbs: { small: string };
}

export interface WallhavenPage {
  data: WallhavenWallpaper[];
  meta: { current_page: number; last_page: number };
}
