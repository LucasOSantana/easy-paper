//! HTTP client shared by every online source.

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::LazyLock;

use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
use ureq::Agent;

pub static AGENT: LazyLock<Agent> = LazyLock::new(|| {
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

/// Streams `url` into `target`, through a `.part` file so an interrupted
/// download never leaves a broken image in the wallpaper folder.
pub fn download_to_file(
    url: &str,
    target: &Path,
    describe_error: fn(ureq::Error) -> String,
) -> Result<(), String> {
    let mut response = AGENT.get(url).call().map_err(describe_error)?;
    let partial = target.with_extension("part");

    let saved = File::create(&partial)
        .and_then(|mut file| io::copy(&mut response.body_mut().as_reader(), &mut file))
        .and_then(|_| fs::rename(&partial, target));

    if let Err(e) = saved {
        let _ = fs::remove_file(&partial);
        return Err(format!("Failed to save wallpaper: {}", e));
    }

    Ok(())
}
