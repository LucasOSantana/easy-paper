//! Unsplash and Pexels, used with the user's own API key.
//!
//! API rules honored here:
//! - Unsplash: attribution links carry `utm_source`/`utm_medium`, and the
//!   photo's `download_location` is called whenever a photo is downloaded.
//! - Pexels: photographer and Pexels are credited in the UI.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::models::{StockPage, StockPhoto, StockProvider, StockSearch};
use crate::services::http_client::{download_to_file, AGENT};

const UNSPLASH_API: &str = "https://api.unsplash.com";
const PEXELS_API: &str = "https://api.pexels.com/v1";
const UNSPLASH_IMAGES: &str = "https://images.unsplash.com/";
const PEXELS_IMAGES: &str = "https://images.pexels.com/";
const UNSPLASH_REFERRAL: &str = "utm_source=easy_paper&utm_medium=referral";

const PER_PAGE: u32 = 24;
/// Downloads are capped at 4K on their longest side.
const MAX_DIMENSION: u32 = 3840;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Makes the cheapest authenticated request to check that `key` works.
pub fn verify_key(provider: StockProvider, key: &str) -> Result<(), String> {
    let request = match provider {
        StockProvider::Unsplash => unsplash_get(&format!("{UNSPLASH_API}/photos"), key),
        StockProvider::Pexels => pexels_get(&format!("{PEXELS_API}/curated"), key),
    };

    request
        .query("per_page", "1")
        .call()
        .map(|_| ())
        .map_err(|e| describe_error(provider, e))
}

pub fn search(search: &StockSearch, key: &str) -> Result<StockPage, String> {
    match search.provider {
        StockProvider::Unsplash => search_unsplash(search, key),
        StockProvider::Pexels => search_pexels(search, key),
    }
    .map_err(|e| describe_error(search.provider, e))
}

/// Saves `photo` into `folder` as `<provider>-<id>.<ext>`. An already
/// downloaded photo is not fetched again.
pub fn download(
    provider: StockProvider,
    photo: &StockPhoto,
    key: &str,
    folder: &Path,
) -> Result<PathBuf, String> {
    let image_host = match provider {
        StockProvider::Unsplash => UNSPLASH_IMAGES,
        StockProvider::Pexels => PEXELS_IMAGES,
    };
    if !photo.download_url.starts_with(image_host) {
        return Err(format!("Invalid {} image URL", provider.name()));
    }

    let id: String = photo
        .id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if id.is_empty() {
        return Err(format!("Invalid {} photo id", provider.name()));
    }

    let target = folder.join(format!("{}-{}.{}", provider.id(), id, file_extension(provider, &photo.download_url)));
    if target.exists() {
        return Ok(target);
    }

    if provider == StockProvider::Unsplash {
        track_unsplash_download(photo, key);
    }

    download_to_file(&photo.download_url, &target, describe_image_error)?;
    Ok(target)
}

// ---------------------------------------------------------------------------
// Unsplash
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct UnsplashPhoto {
    id: String,
    width: u32,
    height: u32,
    urls: UnsplashUrls,
    links: UnsplashLinks,
    user: UnsplashUser,
}

#[derive(Deserialize)]
struct UnsplashUrls {
    raw: String,
    small: String,
}

#[derive(Deserialize)]
struct UnsplashLinks {
    html: String,
    download_location: String,
}

#[derive(Deserialize)]
struct UnsplashUser {
    name: String,
    links: UnsplashUserLinks,
}

#[derive(Deserialize)]
struct UnsplashUserLinks {
    html: String,
}

#[derive(Deserialize)]
struct UnsplashSearchResponse {
    total_pages: u32,
    results: Vec<UnsplashPhoto>,
}

fn unsplash_get(url: &str, key: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
    AGENT
        .get(url)
        .header("Authorization", format!("Client-ID {}", key))
        .header("Accept-Version", "v1")
}

fn search_unsplash(search: &StockSearch, key: &str) -> Result<StockPage, ureq::Error> {
    let page = search.page.max(1);
    let query = search.query.trim();

    // No search term: browse Unsplash's "Wallpapers" topic.
    let url = if query.is_empty() {
        format!("{UNSPLASH_API}/topics/wallpapers/photos")
    } else {
        format!("{UNSPLASH_API}/search/photos")
    };

    let mut request = unsplash_get(&url, key)
        .query("page", page.to_string())
        .query("per_page", PER_PAGE.to_string());
    if !query.is_empty() {
        request = request.query("query", query);
    }
    if let Some(orientation) = orientation_filter(&search.orientation) {
        request = request.query("orientation", orientation);
    }

    let mut response = request.call()?;

    let (photos, has_next) = if query.is_empty() {
        let photos: Vec<UnsplashPhoto> = response.body_mut().read_json()?;
        let has_next = photos.len() as u32 == PER_PAGE;
        (photos, has_next)
    } else {
        let result: UnsplashSearchResponse = response.body_mut().read_json()?;
        (result.results, page < result.total_pages)
    };

    Ok(StockPage {
        photos: photos.into_iter().map(normalize_unsplash).collect(),
        page,
        has_next,
    })
}

fn normalize_unsplash(photo: UnsplashPhoto) -> StockPhoto {
    let download_url = format!(
        "{}{}fm=jpg&q=85&fit=max&w={MAX_DIMENSION}&h={MAX_DIMENSION}",
        photo.urls.raw,
        query_separator(&photo.urls.raw),
    );

    StockPhoto {
        id: photo.id,
        width: photo.width,
        height: photo.height,
        thumb_url: photo.urls.small,
        download_url,
        page_url: with_query(&photo.links.html, UNSPLASH_REFERRAL),
        author_name: photo.user.name,
        author_url: with_query(&photo.user.links.html, UNSPLASH_REFERRAL),
        download_location: Some(photo.links.download_location),
    }
}

/// Required by the Unsplash API guidelines. A failure here must not block the
/// user's download, so errors are only logged.
fn track_unsplash_download(photo: &StockPhoto, key: &str) {
    let Some(location) = photo
        .download_location
        .as_deref()
        .filter(|location| location.starts_with(UNSPLASH_API))
    else {
        return;
    };

    if let Err(e) = unsplash_get(location, key).call() {
        eprintln!("[unsplash] Download tracking failed: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Pexels
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PexelsResponse {
    photos: Vec<PexelsPhoto>,
    next_page: Option<String>,
}

#[derive(Deserialize)]
struct PexelsPhoto {
    id: u64,
    width: u32,
    height: u32,
    url: String,
    photographer: String,
    photographer_url: String,
    src: PexelsSources,
}

#[derive(Deserialize)]
struct PexelsSources {
    original: String,
    medium: String,
}

fn pexels_get(url: &str, key: &str) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
    AGENT.get(url).header("Authorization", key)
}

fn search_pexels(search: &StockSearch, key: &str) -> Result<StockPage, ureq::Error> {
    let page = search.page.max(1);
    let query = search.query.trim();

    // No search term: Pexels' hand-picked "curated" photos.
    let mut request = if query.is_empty() {
        pexels_get(&format!("{PEXELS_API}/curated"), key)
    } else {
        let mut request = pexels_get(&format!("{PEXELS_API}/search"), key).query("query", query);
        if let Some(orientation) = orientation_filter(&search.orientation) {
            request = request.query("orientation", orientation);
        }
        request
    };
    request = request
        .query("page", page.to_string())
        .query("per_page", PER_PAGE.to_string());

    let result: PexelsResponse = request.call()?.body_mut().read_json()?;

    Ok(StockPage {
        has_next: result.next_page.is_some(),
        photos: result.photos.into_iter().map(normalize_pexels).collect(),
        page,
    })
}

fn normalize_pexels(photo: PexelsPhoto) -> StockPhoto {
    let cap = if photo.width >= photo.height && photo.width > MAX_DIMENSION {
        format!("&w={MAX_DIMENSION}")
    } else if photo.height > photo.width && photo.height > MAX_DIMENSION {
        format!("&h={MAX_DIMENSION}")
    } else {
        String::new()
    };

    StockPhoto {
        id: photo.id.to_string(),
        width: photo.width,
        height: photo.height,
        thumb_url: photo.src.medium,
        download_url: format!("{}?auto=compress&cs=tinysrgb{}", photo.src.original, cap),
        page_url: photo.url,
        author_name: photo.photographer,
        author_url: photo.photographer_url,
        download_location: None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn orientation_filter(value: &str) -> Option<&'static str> {
    match value {
        "landscape" => Some("landscape"),
        "portrait" => Some("portrait"),
        _ => None,
    }
}

fn query_separator(url: &str) -> char {
    if url.contains('?') { '&' } else { '?' }
}

fn with_query(url: &str, query: &str) -> String {
    format!("{}{}{}", url, query_separator(url), query)
}

fn file_extension(provider: StockProvider, url: &str) -> &'static str {
    if provider == StockProvider::Unsplash {
        return "jpg"; // requested with fm=jpg
    }

    let path = url.split('?').next().unwrap_or(url);
    match path.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
        Some("png") => "png",
        Some("webp") => "webp",
        Some("jpeg") => "jpeg",
        _ => "jpg",
    }
}

fn describe_error(provider: StockProvider, error: ureq::Error) -> String {
    let name = provider.name();
    match error {
        ureq::Error::StatusCode(401) => {
            format!("{} rejected the API key. Check it in the {} tab and try again.", name, name)
        }
        ureq::Error::StatusCode(403) if provider == StockProvider::Unsplash => {
            "Unsplash refused the request. Demo keys allow 50 requests per hour, so wait a bit and try again."
                .to_string()
        }
        ureq::Error::StatusCode(429) => format!("{} rate limit reached. Try again later.", name),
        error => format!("{} request failed: {}", name, error),
    }
}

fn describe_image_error(error: ureq::Error) -> String {
    format!("Image download failed: {}", error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_referral_to_unsplash_links() {
        assert_eq!(
            with_query("https://unsplash.com/@someone", UNSPLASH_REFERRAL),
            "https://unsplash.com/@someone?utm_source=easy_paper&utm_medium=referral"
        );
        assert_eq!(
            with_query("https://unsplash.com/photos/x?lang=en", UNSPLASH_REFERRAL),
            "https://unsplash.com/photos/x?lang=en&utm_source=easy_paper&utm_medium=referral"
        );
    }

    #[test]
    fn caps_large_pexels_downloads_on_longest_side() {
        let photo = |width, height| PexelsPhoto {
            id: 1,
            width,
            height,
            url: String::new(),
            photographer: String::new(),
            photographer_url: String::new(),
            src: PexelsSources {
                original: "https://images.pexels.com/photos/1/pexels-photo-1.jpeg".to_string(),
                medium: String::new(),
            },
        };

        assert!(normalize_pexels(photo(6000, 4000)).download_url.ends_with("&w=3840"));
        assert!(normalize_pexels(photo(4000, 6000)).download_url.ends_with("&h=3840"));
        assert!(normalize_pexels(photo(1920, 1080)).download_url.ends_with("cs=tinysrgb"));
    }

    #[test]
    fn rejects_downloads_from_other_hosts() {
        let photo = StockPhoto {
            id: "abc".to_string(),
            width: 1,
            height: 1,
            thumb_url: String::new(),
            download_url: "https://evil.example.com/a.jpg".to_string(),
            page_url: String::new(),
            author_name: String::new(),
            author_url: String::new(),
            download_location: None,
        };
        let folder = std::env::temp_dir();
        assert!(download(StockProvider::Unsplash, &photo, "key", &folder).is_err());
        assert!(download(StockProvider::Pexels, &photo, "key", &folder).is_err());
    }

    #[test]
    fn keeps_pexels_file_extension() {
        assert_eq!(file_extension(StockProvider::Pexels, "https://images.pexels.com/p/x.jpeg?auto=compress"), "jpeg");
        assert_eq!(file_extension(StockProvider::Pexels, "https://images.pexels.com/p/x.png"), "png");
        assert_eq!(file_extension(StockProvider::Unsplash, "https://images.unsplash.com/photo-1?ixid=1"), "jpg");
    }
}
