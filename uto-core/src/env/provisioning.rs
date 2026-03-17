use std::io::Write;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{UtoError, UtoResult};

// ---------------------------------------------------------------------------
// Chrome for Testing API types
// ---------------------------------------------------------------------------

/// Root response from the Chrome for Testing "known-good-versions" endpoint.
#[derive(Debug, Deserialize)]
struct CftVersionsResponse {
    versions: Vec<CftVersion>,
}

/// A single entry in the Chrome for Testing version list.
#[derive(Debug, Deserialize)]
struct CftVersion {
    version: String,
    downloads: CftDownloads,
}

/// Available download artifacts for a given Chrome version.
#[derive(Debug, Deserialize)]
struct CftDownloads {
    chromedriver: Option<Vec<CftDownload>>,
}

/// A single platform-specific download artifact.
#[derive(Debug, Deserialize)]
struct CftDownload {
    platform: String,
    url: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns the path to a `chromedriver` binary that matches `chrome_version`.
///
/// If a matching binary is already cached in `.uto/cache/chromedriver/`, it is
/// returned directly.  Otherwise the binary is downloaded from the Chrome for
/// Testing JSON API, extracted into the cache and the path is returned.
pub async fn find_or_provision_chromedriver(chrome_version: &str) -> UtoResult<PathBuf> {
    let cache_path = chromedriver_cache_path(chrome_version)?;

    if cache_path.exists() {
        log::info!("Using cached chromedriver at {}", cache_path.display());
        return Ok(cache_path);
    }

    log::info!(
        "ChromeDriver not found for Chrome {}; downloading...",
        chrome_version
    );
    download_chromedriver(chrome_version, &cache_path).await?;
    Ok(cache_path)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the expected path of the cached `chromedriver` binary.
pub(crate) fn chromedriver_cache_path(chrome_version: &str) -> UtoResult<PathBuf> {
    let major = major_version(chrome_version);
    let base = dirs::home_dir()
        .ok_or_else(|| UtoError::Internal("Could not determine home directory".to_string()))?
        .join(".uto")
        .join("cache")
        .join("chromedriver")
        .join(major);

    Ok(base.join(chromedriver_binary_name()))
}

/// Extracts the major version component from a full version string.
/// e.g. `"124.0.6367.60"` → `"124"`.
fn major_version(version: &str) -> &str {
    version.split('.').next().unwrap_or(version)
}

/// Returns the platform-specific binary name for `chromedriver`.
fn chromedriver_binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "chromedriver.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "chromedriver"
    }
}

/// Returns the platform string used by the Chrome for Testing API.
fn cft_platform() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "linux64"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "mac-x64"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "mac-arm64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "win64"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    {
        "win32"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86"),
    )))]
    {
        "linux64"
    }
}

/// Downloads and extracts a `chromedriver` matching `chrome_version` to
/// `dest_path`.
async fn download_chromedriver(chrome_version: &str, dest_path: &PathBuf) -> UtoResult<()> {
    let major = major_version(chrome_version);
    let platform = cft_platform();

    // Fetch the version manifest from the Chrome for Testing API.
    let url = "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";
    let response: CftVersionsResponse = reqwest::get(url).await?.json().await?;

    // Find the best matching version for this major release.
    let download_url = response
        .versions
        .iter()
        .filter(|v| major_version(&v.version) == major)
        .filter_map(|v| v.downloads.chromedriver.as_ref())
        .flatten()
        .find(|d| d.platform == platform)
        .map(|d| d.url.clone())
        .ok_or_else(|| {
            UtoError::Internal(format!(
                "No chromedriver download found for Chrome {} on {}",
                chrome_version, platform
            ))
        })?;

    log::info!("Downloading chromedriver from {}", download_url);

    // Download the ZIP archive into memory.
    let zip_bytes = reqwest::get(&download_url).await?.bytes().await?;

    // Create the destination directory.
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Extract the `chromedriver` binary from the archive.
    extract_chromedriver_from_zip(&zip_bytes, dest_path)?;

    // Make the binary executable on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest_path, std::fs::Permissions::from_mode(0o755))?;
    }

    log::info!("chromedriver installed at {}", dest_path.display());
    Ok(())
}

/// Extracts the `chromedriver` binary from a ZIP archive held in `bytes`.
pub(crate) fn extract_chromedriver_from_zip(bytes: &[u8], dest: &PathBuf) -> UtoResult<()> {
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| UtoError::Internal(format!("Failed to open ZIP archive: {e}")))?;

    let binary_name = chromedriver_binary_name();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| UtoError::Internal(format!("ZIP read error: {e}")))?;

        // The archive nests the binary inside a platform sub-directory.
        // Match any entry whose file name (last path component) equals the binary name.
        let entry_name = file.name().to_string();
        let file_name = std::path::Path::new(&entry_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if file_name == binary_name {
            let mut out = std::fs::File::create(dest)?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buf)?;
            out.write_all(&buf)?;
            return Ok(());
        }
    }

    Err(UtoError::Internal(format!(
        "'{binary_name}' not found inside the downloaded archive"
    )))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // major_version
    // -----------------------------------------------------------------------

    #[test]
    fn major_version_extraction() {
        assert_eq!(major_version("124.0.6367.60"), "124");
        assert_eq!(major_version("120"), "120");
        assert_eq!(major_version(""), "");
    }

    // -----------------------------------------------------------------------
    // chromedriver_cache_path
    // -----------------------------------------------------------------------

    #[test]
    fn cache_path_contains_major_version() {
        let path = chromedriver_cache_path("124.0.6367.60").expect("cache path");
        // The path must contain "124" as a directory segment.
        assert!(
            path.components().any(|c| c.as_os_str() == "124"),
            "expected '124' segment in {path:?}"
        );
        // And the final component must be the chromedriver binary name.
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some(chromedriver_binary_name()),
        );
    }

    // -----------------------------------------------------------------------
    // extract_chromedriver_from_zip
    // -----------------------------------------------------------------------

    /// Build a minimal in-memory ZIP that contains a fake `chromedriver`
    /// binary nested inside a platform sub-directory (matching the real
    /// Chrome for Testing archive layout).
    fn make_fake_chromedriver_zip() -> Vec<u8> {
        use std::io::Write;

        let cursor = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::FileOptions::default();

        // Mirror the real archive layout: <platform>/<binary>
        let entry = format!("chromedriver-linux64/{}", chromedriver_binary_name());
        zip.start_file(entry, options).expect("zip start_file");
        zip.write_all(b"fake chromedriver binary content")
            .expect("zip write");

        zip.finish().expect("zip finish").into_inner()
    }

    #[test]
    fn extract_chromedriver_from_zip_writes_binary_to_dest() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join(chromedriver_binary_name());

        let zip_bytes = make_fake_chromedriver_zip();
        extract_chromedriver_from_zip(&zip_bytes, &dest).expect("extract");

        assert!(
            dest.exists(),
            "chromedriver binary should exist after extraction"
        );
        let content = std::fs::read(&dest).expect("read extracted file");
        assert_eq!(content, b"fake chromedriver binary content");
    }

    #[test]
    fn extract_chromedriver_from_zip_errors_when_binary_missing() {
        use std::io::Write;

        // Build a ZIP that does NOT contain the chromedriver binary.
        let cursor = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let options = zip::write::FileOptions::default();
        zip.start_file("unrelated-file.txt", options)
            .expect("zip start_file");
        zip.write_all(b"nothing useful").expect("zip write");
        let zip_bytes = zip.finish().expect("zip finish").into_inner();

        let dir = tempfile::tempdir().expect("tempdir");
        let dest = dir.path().join(chromedriver_binary_name());

        let result = extract_chromedriver_from_zip(&zip_bytes, &dest);
        assert!(
            result.is_err(),
            "should fail when binary is absent from archive"
        );
    }

    // -----------------------------------------------------------------------
    // find_or_provision_chromedriver — cache-hit (no network required)
    // -----------------------------------------------------------------------

    /// If the chromedriver binary already exists at the expected cache path,
    /// `find_or_provision_chromedriver` must return that path immediately
    /// without attempting any network request.
    #[tokio::test]
    async fn find_or_provision_chromedriver_returns_cached_path() {
        // Determine the cache path for a dummy version.
        let version = "124.0.6367.60";
        let cache_path = chromedriver_cache_path(version).expect("cache path");

        // Pre-create the binary so the cache-hit branch is taken.
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir cache dir");
        }
        std::fs::write(&cache_path, b"stub").expect("write stub chromedriver");

        let result = find_or_provision_chromedriver(version).await;

        // Clean up before asserting so a failing assert doesn't leave garbage.
        let _ = std::fs::remove_file(&cache_path);

        let returned = result.expect("should return cached path");
        assert_eq!(returned, cache_path);
    }
}
