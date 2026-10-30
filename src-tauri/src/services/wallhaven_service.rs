use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::Agent;
use crate::models::{WallhavenPage, WallhavenSearch};

const SEARCH_URL: &str = "https://wallhaven.cc/api/v1/search";
const IMAGE_HOST: &str = "https://w.wallhaven.cc/";

static AGENT: LazyLock<Agent> = LazyLock::new(|| {
    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();

    Agent::config_builder()
        .tls_config(tls)
        .user_agent(concat!("easy-paper/", env!("CARGO_PKG_VERSION")))
        .build()
        .new_agent()
});

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
        .ok_or("Invalid Wallhaven image URL")?;

    let target = folder.join(file_name);
    if target.exists() {
        return Ok(target);
    }

    let mut response = AGENT.get(url).call().map_err(describe_error)?;
    let partial = target.with_extension("part");
    let saved = File::create(&partial)
        .and_then(|mut file| io::copy(&mut response.body_mut().as_reader(), &mut file))
        .and_then(|_| fs::rename(&partial, &target));

    if let Err(e) = saved {
        let _ = fs::remove_file(&partial);
        return Err(format!("Failed to save wallpaper: {}", e));
    }

    Ok(target)
}

fn describe_error(error: ureq::Error) -> String {
    match error {
        ureq::Error::StatusCode(429) => "Wallhaven rate limit reached. Try again in a minute.".to_string(),
        error => format!("Wallhaven request failed: {}", error),
    }
}
