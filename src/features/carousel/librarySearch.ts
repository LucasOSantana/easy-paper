import { ImageItem } from "../../types/wallpaper";

/** Lowercase, without accents, with separators (`_`, `-`, `.`) as spaces. */
export function normalizeForSearch(text: string): string {
  return text
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[_\-.]+/g, " ");
}

/**
 * Wallpapers whose file name contains every word of `query` (in any order,
 * ignoring case and accents), optionally only favorites.
 */
export function filterLibrary(images: ImageItem[], query: string, favoritesOnly: boolean): ImageItem[] {
  const words = normalizeForSearch(query).split(/\s+/).filter(Boolean);

  return images.filter((image) => {
    if (favoritesOnly && !image.favorite) return false;
    if (words.length === 0) return true;
    const name = normalizeForSearch(image.name);
    return words.every((word) => name.includes(word));
  });
}
