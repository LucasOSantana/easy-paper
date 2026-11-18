use std::path::{Path, PathBuf};
use crate::models::{WallhavenPage, WallhavenSearch};
use crate::services::http_client::{download_to_file, AGENT};

const SEARCH_URL: &str = "https://wallhaven.cc/api/v1/search";
const IMAGE_HOST: &str = "https://w.wallhaven.cc/";

pub fn search(search: &WallhavenSearch, min_resolution: Option<(u32, u32)>) -> Result<WallhavenPage, String> {
    let mut request = AGENT
        .get(SEARCH_URL)
        .query("q", search.query.trim())
        .query("categories", "111")
        .query("purity", "100")
        .query("sorting", &search.sorting)
        .query("page", search.page.to_string());

    if !search.ratios.is_empty() {
        request = request.query("ratios", &search.ratios);
    }
    if let Some((width, height)) = min_resolution {
        request = request.query("atleast", format!("{}x{}", width, height));
    }

    request
        .call()
        .map_err(describe_error)?
        .body_mut()
        .read_json()
        .map_err(describe_error)
}

pub fn download(url: &str, folder: &Path) -> Result<PathBuf, String> {
    let file_name = url
        .strip_prefix(IMAGE_HOST)
        .and_then(|path| path.rsplit('/').next())
        .filter(|name| !name.is_empty() && !name.contains(['\\', ':']) && !name.starts_with('.'))
        .ok_or_else(|| crate::tr!(en: "Invalid Wallhaven image URL", pt: "URL de imagem do Wallhaven inválida"))?;

    let target = folder.join(file_name);
    if target.exists() {
        return Ok(target);
    }

    download_to_file(url, &target, describe_error)?;
    Ok(target)
}

fn describe_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(429) => crate::tr!(
            en: "Wallhaven rate limit reached. Try again in a minute.",
            pt: "Limite de pedidos do Wallhaven atingido. Tente de novo em um minuto.",
        ),
        error => crate::tr!(
            en: format!("Wallhaven request failed: {}", error),
            pt: format!("O pedido ao Wallhaven falhou: {}", error),
        ),
    }
}
