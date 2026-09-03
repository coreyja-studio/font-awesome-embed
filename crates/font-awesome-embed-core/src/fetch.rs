use std::fs;
use std::path::PathBuf;

#[cfg(not(feature = "test-icons"))]
use serde::Deserialize;

/// Font Awesome release used when `FA_VERSION` is not set.
///
/// The API's `version: "latest"` resolves to a 5.x release, so we pin a
/// modern default explicitly. Override with the `FA_VERSION` env var.
pub const FA_DEFAULT_VERSION: &str = "7.3.0";

/// Bumped whenever post-processing changes, so stale cache entries
/// (which store *processed* SVGs) are never reused across formats.
const CACHE_FORMAT_VERSION: u32 = 1;

/// Runtime fetch mode: real API calls or static placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchMode {
    Real,
    Placeholder,
}

/// Cache namespace for a fetch mode.
///
/// Placeholder and real SVGs must never share cache entries. When the
/// `test-icons` feature strips the real transport, nothing reaching the
/// cache is genuinely real either — so `Real` gets its own `test-icons`
/// namespace in that build, and a production build can never read an
/// entry a test build seeded.
fn cache_mode(mode: FetchMode) -> &'static str {
    match mode {
        FetchMode::Placeholder => "placeholders",
        #[cfg(feature = "test-icons")]
        FetchMode::Real => "test-icons",
        #[cfg(not(feature = "test-icons"))]
        FetchMode::Real => "real",
    }
}

/// Generate a placeholder SVG (no network, no token).
/// Extracted from send_graphql_request's #[cfg(feature = "test-icons")] branch.
fn placeholder_svg(name: &str, style: &str) -> String {
    let style_lower = style.to_lowercase();
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" data-icon="{name}" data-style="{style_lower}"><rect width="512" height="512" opacity="0.2"/></svg>"#
    )
}

/// Validate a Font Awesome icon name.
///
/// Icon names are slugs like `house` or `arrow-up-right-from-square`:
/// lowercase ASCII letters, digits, and hyphens. This is also a security
/// boundary — the name is used in a filesystem cache path, so anything
/// outside the slug charset (e.g. `../`) must be rejected.
pub fn validate_icon_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("icon name must not be empty".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(format!(
            "invalid icon name `{name}`: icon names may only contain \
             lowercase letters, digits, and hyphens"
        ));
    }
    Ok(())
}

/// Validate a Font Awesome release version (from `FA_VERSION`).
///
/// Like icon names, the version is part of the cache path, so restrict it
/// to a safe charset.
fn validate_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("FA_VERSION must not be empty".to_string());
    }
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err(format!(
            "invalid FA_VERSION `{version}`: versions may only contain \
             alphanumerics, dots, and hyphens"
        ));
    }
    Ok(())
}

fn fa_version() -> Result<String, String> {
    let version = std::env::var("FA_VERSION").unwrap_or_else(|_| FA_DEFAULT_VERSION.to_string());
    validate_version(&version)?;
    Ok(version)
}

/// Get an icon SVG, checking cache first, then fetching from the API.
///
/// When `mode` is `FetchMode::Placeholder`, or when the `test-icons` feature
/// is enabled (which strips real transport), placeholder SVGs are used
/// without any network call. Caching and post-processing run in all modes.
pub fn get_icon(name: &str, family: &str, style: &str, mode: FetchMode) -> Result<String, String> {
    use crate::postprocess;

    validate_icon_name(name)?;

    let version = fa_version()?;

    if let Some(cached) = cache_get(&version, family, style, name, mode) {
        return Ok(cached);
    }

    let raw_svg = fetch_icon(name, family, style, &version, mode)?;

    let processed = postprocess::process_svg(&raw_svg);

    cache_set(&version, family, style, name, &processed, mode);

    Ok(processed)
}

fn fetch_icon(
    name: &str,
    family: &str,
    style: &str,
    version: &str,
    mode: FetchMode,
) -> Result<String, String> {
    match mode {
        FetchMode::Placeholder => Ok(placeholder_svg(name, style)),
        #[cfg(not(feature = "test-icons"))]
        FetchMode::Real => fetch_from_api(name, family, style, version),
        // Real transport is not compiled when test-icons is on. Fail loudly
        // rather than silently substituting placeholders: callers that ask
        // for `Real` (e.g. fa-vendor without `--placeholders`) would
        // otherwise emit grey rectangles as if they were licensed icons.
        #[cfg(feature = "test-icons")]
        FetchMode::Real => {
            let _ = (name, family, style, version);
            Err("cannot fetch real icons: this binary was built with the \
                 `test-icons` feature, which strips the HTTP transport. \
                 Rebuild without `test-icons`, or request placeholders \
                 explicitly."
                .to_string())
        }
    }
}

/// Cache root, in priority order: `FA_CACHE_DIR` (explicit, e.g. a
/// directory committed to the consuming repo so CI and Docker builds need
/// neither network nor token), then `OUT_DIR`, then a shared temp dir.
///
/// `FA_CACHE_DIR` should be an absolute path — proc macros make no
/// guarantee about the working directory. Set it via `[env]` in
/// `.cargo/config.toml` with `relative = true` to anchor it to the repo.
fn cache_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("FA_CACHE_DIR")
        && !dir.is_empty()
    {
        return PathBuf::from(dir);
    }
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        return PathBuf::from(out_dir).join("fa-cache");
    }
    std::env::temp_dir().join("font-awesome-embed-cache")
}

/// Cache path keyed by format version, FA release, family/style, and name —
/// so bumping the pinned release or the post-processing format never reuses
/// stale entries. All components are charset-validated or static enum values.
fn cache_path(version: &str, family: &str, style: &str, name: &str, mode: FetchMode) -> PathBuf {
    cache_dir()
        .join(format!("v{CACHE_FORMAT_VERSION}"))
        .join(cache_mode(mode))
        .join(version)
        .join(format!("{family}-{style}"))
        .join(format!("{name}.svg"))
}

fn cache_get(
    version: &str,
    family: &str,
    style: &str,
    name: &str,
    mode: FetchMode,
) -> Option<String> {
    fs::read_to_string(cache_path(version, family, style, name, mode)).ok()
}

fn cache_set(version: &str, family: &str, style: &str, name: &str, svg: &str, mode: FetchMode) {
    let path = cache_path(version, family, style, name, mode);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, svg);
}

// --- API response types (only compiled when test-icons is off) ---

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<GraphQLError>>,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct GraphQLData {
    release: Option<ReleaseData>,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct ReleaseData {
    icon: Option<IconData>,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct IconData {
    svgs: Vec<SvgEntry>,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct SvgEntry {
    html: String,
}

#[cfg(not(feature = "test-icons"))]
#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

// --- HTTP transport (only compiled when test-icons is off) ---

#[cfg(not(feature = "test-icons"))]
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[cfg(not(feature = "test-icons"))]
fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build()
        .new_agent()
}

/// Retry transient API failures (rate limits, server errors) with backoff.
/// A compile expanding many icons in parallel with other CI jobs can trip
/// Font Awesome's rate limiting; a brief pause resolves it.
#[cfg(not(feature = "test-icons"))]
fn retry_transient<T>(mut call: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    let mut delay = std::time::Duration::from_secs(1);
    let mut last_err = String::new();
    for attempt in 0..4 {
        match call() {
            Ok(value) => return Ok(value),
            Err(e) => {
                let transient = e.contains("status: 429") || e.contains("status: 5");
                last_err = e;
                if !transient || attempt == 3 {
                    break;
                }
                std::thread::sleep(delay);
                delay *= 2;
            }
        }
    }
    Err(last_err)
}

/// Exchange an API token for an access token.
///
/// The access token is cached for the life of the process — without this,
/// every `fa!()` expansion performs its own token exchange and a compile
/// with many icons rate-limits the token endpoint (HTTP 429).
#[cfg(not(feature = "test-icons"))]
fn get_access_token(api_token: &str) -> Result<String, String> {
    static ACCESS_TOKEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

    let mut cached = ACCESS_TOKEN.lock().expect("access token lock poisoned");
    if let Some(token) = cached.as_ref() {
        return Ok(token.clone());
    }

    let token = retry_transient(|| {
        let resp: TokenResponse = http_agent()
            .post("https://api.fontawesome.com/token")
            .header("Authorization", &format!("Bearer {api_token}"))
            .send_empty()
            .map_err(|e| format!("token exchange request failed: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse token response: {e}"))?;
        Ok(resp.access_token)
    })?;

    *cached = Some(token.clone());
    Ok(token)
}

/// Send a GraphQL request to the Font Awesome API.
#[cfg(not(feature = "test-icons"))]
fn send_graphql_request(
    access_token: &str,
    body: &serde_json::Value,
) -> Result<GraphQLResponse, String> {
    retry_transient(|| {
        let resp: GraphQLResponse = http_agent()
            .post("https://api.fontawesome.com")
            .header("Authorization", &format!("Bearer {access_token}"))
            .send_json(body)
            .map_err(|e| format!("GraphQL request failed: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse GraphQL response: {e}"))?;

        Ok(resp)
    })
}

/// GraphQL query for one icon's SVG. All caller-supplied values are passed
/// as GraphQL variables — never interpolated into the query text — so a
/// hostile icon name cannot inject query syntax.
#[cfg(not(feature = "test-icons"))]
const ICON_QUERY: &str = "\
query FaIcon($version: String!, $name: String!, $family: Family!, $style: Style!) {
  release(version: $version) {
    icon(name: $name) {
      svgs(filter: { familyStyles: [{ family: $family, style: $style }] }) {
        html
      }
    }
  }
}";

/// Fetch an icon SVG from the Font Awesome GraphQL API.
///
/// Builds the query, authenticates, sends the request, and extracts
/// the SVG from the response.
#[cfg(not(feature = "test-icons"))]
fn fetch_from_api(name: &str, family: &str, style: &str, version: &str) -> Result<String, String> {
    let api_token = match std::env::var("FONT_AWESOME_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            return Err(
                "FONT_AWESOME_TOKEN environment variable not set. \
                 Set it to your Font Awesome API token, or use the `test-icons` feature for development."
                    .to_string(),
            );
        }
    };

    let access_token = get_access_token(&api_token)?;

    let body = serde_json::json!({
        "query": ICON_QUERY,
        "variables": {
            "version": version,
            "name": name,
            "family": family,
            "style": style,
        },
    });

    let resp = send_graphql_request(&access_token, &body)?;

    if let Some(errors) = resp.errors {
        let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        return Err(format!("GraphQL errors: {}", msgs.join("; ")));
    }

    let data = resp.data.ok_or("no data in GraphQL response")?;
    let release = data
        .release
        .ok_or_else(|| format!("Font Awesome release `{version}` not found"))?;
    let icon = release
        .icon
        .ok_or_else(|| format!("icon `{name}` not found in release {version}"))?;
    let svg = icon.svgs.into_iter().next().ok_or_else(|| {
        format!(
            "no SVG for icon `{name}` with family `{family}` and style `{style}` \
             in release {version} — not every family ships every style \
             (e.g. pixel is regular-only, notdog is solid-only)"
        )
    })?;

    Ok(svg.html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_icon_names() {
        for name in ["house", "arrow-up-right-from-square", "42-group", "00"] {
            assert!(validate_icon_name(name).is_ok(), "expected `{name}` valid");
        }
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_icon_name("../../evil").is_err());
        assert!(validate_icon_name("solid/../../x").is_err());
        assert!(validate_icon_name("..").is_err());
    }

    #[test]
    fn rejects_graphql_metacharacters() {
        assert!(validate_icon_name(r#"hou"se"#).is_err());
        assert!(validate_icon_name("a{b}").is_err());
        assert!(validate_icon_name("a b").is_err());
        assert!(validate_icon_name("Upper").is_err());
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_icon_name("").is_err());
    }

    /// Regression: placeholder SVGs must never be reachable from a cache
    /// path a real build would read. Before this was enforced, a
    /// `test-icons` build writing under `FetchMode::Real` seeded the
    /// `real/` namespace with grey rectangles, and a later production
    /// build served them without a token and without erroring.
    #[test]
    fn cache_namespaces_are_disjoint_across_modes() {
        let real = cache_path("7.3.0", "CLASSIC", "SOLID", "house", FetchMode::Real);
        let placeholder = cache_path("7.3.0", "CLASSIC", "SOLID", "house", FetchMode::Placeholder);
        assert_ne!(real, placeholder);
    }

    /// A build without real transport must refuse `FetchMode::Real` rather
    /// than quietly downgrading to placeholders.
    #[cfg(feature = "test-icons")]
    #[test]
    fn real_mode_errors_without_transport() {
        let err = fetch_icon("house", "CLASSIC", "SOLID", "7.3.0", FetchMode::Real)
            .expect_err("test-icons build must not produce a real icon");
        assert!(err.contains("test-icons"), "unexpected error: {err}");
    }

    #[test]
    fn version_charset() {
        assert!(validate_version("7.3.0").is_ok());
        assert!(validate_version("7.0.0-beta1").is_ok());
        assert!(validate_version("../../etc").is_err());
        assert!(validate_version("").is_err());
    }

    #[test]
    fn cache_dir_honors_fa_cache_dir_env() {
        // SAFETY: no other test reads or writes this variable.
        unsafe { std::env::set_var("FA_CACHE_DIR", "/custom/cache") };
        let dir = cache_dir();
        unsafe { std::env::remove_var("FA_CACHE_DIR") };
        assert_eq!(dir, PathBuf::from("/custom/cache"));
    }

    #[test]
    fn cache_path_is_versioned() {
        let path = cache_path("7.3.0", "CLASSIC", "SOLID", "house", FetchMode::Real);
        let s = path.to_string_lossy();
        assert!(s.contains(&format!("v{CACHE_FORMAT_VERSION}")));
        assert!(s.contains("7.3.0"));
        assert!(s.contains("CLASSIC-SOLID"));
        // Mode segment, not the literal "real" — under `test-icons` the
        // real-transport namespace is deliberately renamed.
        assert!(s.contains(cache_mode(FetchMode::Real)));
        assert!(s.ends_with("house.svg"));
    }
}
