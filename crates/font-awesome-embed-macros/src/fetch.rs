use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

/// Font Awesome release used when `FA_VERSION` is not set.
///
/// The API's `version: "latest"` resolves to a 5.x release, so we pin a
/// modern default explicitly. Override with the `FA_VERSION` env var.
pub const FA_DEFAULT_VERSION: &str = "7.3.0";

/// Bumped whenever post-processing changes, so stale cache entries
/// (which store *processed* SVGs) are never reused across formats.
const CACHE_FORMAT_VERSION: u32 = 1;

#[cfg(not(feature = "test-icons"))]
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Placeholder and real SVGs must never share cache entries — without a
/// build script OUT_DIR is unset and both modes fall back to the same
/// shared temp dir.
#[cfg(feature = "test-icons")]
const CACHE_MODE: &str = "test-icons";
#[cfg(not(feature = "test-icons"))]
const CACHE_MODE: &str = "real";

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
/// When the `test-icons` feature is enabled, the HTTP calls return fake
/// responses — but caching, query building, response parsing, and
/// post-processing all run the same code path as production.
pub fn get_icon(name: &str, family: &str, style: &str) -> Result<String, String> {
    use crate::postprocess;

    // Defense in depth: the proc macro validates at parse time with a
    // spanned error, but nothing else may call this with a raw name.
    validate_icon_name(name)?;

    let version = fa_version()?;

    if let Some(cached) = cache_get(&version, family, style, name) {
        return Ok(cached);
    }

    // Fetch from API (HTTP calls are faked in test-icons mode)
    let raw_svg = fetch_from_api(name, family, style, &version)?;

    let processed = postprocess::process_svg(&raw_svg);

    cache_set(&version, family, style, name, &processed);

    Ok(processed)
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
fn cache_path(version: &str, family: &str, style: &str, name: &str) -> PathBuf {
    cache_dir()
        .join(format!("v{CACHE_FORMAT_VERSION}"))
        .join(CACHE_MODE)
        .join(version)
        .join(format!("{family}-{style}"))
        .join(format!("{name}.svg"))
}

fn cache_get(version: &str, family: &str, style: &str, name: &str) -> Option<String> {
    fs::read_to_string(cache_path(version, family, style, name)).ok()
}

fn cache_set(version: &str, family: &str, style: &str, name: &str, svg: &str) {
    let path = cache_path(version, family, style, name);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, svg);
}

// --- API response types (always compiled) ---

#[derive(Deserialize)]
#[allow(dead_code)] // Constructed by serde, unused in test-icons mode
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLData {
    release: Option<ReleaseData>,
}

#[derive(Deserialize)]
struct ReleaseData {
    icon: Option<IconData>,
}

#[derive(Deserialize)]
struct IconData {
    svgs: Vec<SvgEntry>,
}

#[derive(Deserialize)]
struct SvgEntry {
    html: String,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

// --- HTTP transport (faked in test-icons mode) ---

#[cfg(not(feature = "test-icons"))]
fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build()
        .new_agent()
}

/// Exchange an API token for an access token.
///
/// In test-icons mode, returns a fake token without making any HTTP call.
fn get_access_token(api_token: &str) -> Result<String, String> {
    #[cfg(feature = "test-icons")]
    {
        let _ = api_token;
        Ok("test-fake-token".to_string())
    }

    #[cfg(not(feature = "test-icons"))]
    {
        let resp: TokenResponse = http_agent()
            .post("https://api.fontawesome.com/token")
            .header("Authorization", &format!("Bearer {api_token}"))
            .send_empty()
            .map_err(|e| format!("token exchange request failed: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse token response: {e}"))?;

        Ok(resp.access_token)
    }
}

/// Send a GraphQL request to the Font Awesome API.
///
/// In test-icons mode, returns a fake response with a placeholder SVG
/// instead of making a network call. The response has the same structure
/// as a real API response, so downstream parsing is exercised in both modes.
fn send_graphql_request(
    access_token: &str,
    body: &serde_json::Value,
    name: &str,
    style: &str,
) -> Result<GraphQLResponse, String> {
    #[cfg(feature = "test-icons")]
    {
        let _ = (access_token, body);
        let style_lower = style.to_lowercase();
        Ok(GraphQLResponse {
            data: Some(GraphQLData {
                release: Some(ReleaseData {
                    icon: Some(IconData {
                        svgs: vec![SvgEntry {
                            html: format!(
                                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" data-icon="{name}" data-style="{style_lower}"><rect width="512" height="512" opacity="0.2"/></svg>"#
                            ),
                        }],
                    }),
                }),
            }),
            errors: None,
        })
    }

    #[cfg(not(feature = "test-icons"))]
    {
        let _ = (name, style);
        let resp: GraphQLResponse = http_agent()
            .post("https://api.fontawesome.com")
            .header("Authorization", &format!("Bearer {access_token}"))
            .send_json(body)
            .map_err(|e| format!("GraphQL request failed: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse GraphQL response: {e}"))?;

        Ok(resp)
    }
}

/// GraphQL query for one icon's SVG. All caller-supplied values are passed
/// as GraphQL variables — never interpolated into the query text — so a
/// hostile icon name cannot inject query syntax.
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
/// the SVG from the response. In test-icons mode, the HTTP calls are
/// faked but query building and response parsing still run.
fn fetch_from_api(name: &str, family: &str, style: &str, version: &str) -> Result<String, String> {
    let api_token = match std::env::var("FONT_AWESOME_TOKEN") {
        Ok(token) => token,
        #[cfg(feature = "test-icons")]
        Err(_) => "test-token".to_string(),
        #[cfg(not(feature = "test-icons"))]
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

    let resp = send_graphql_request(&access_token, &body, name, style)?;

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
        let path = cache_path("7.3.0", "CLASSIC", "SOLID", "house");
        let s = path.to_string_lossy();
        assert!(s.contains(&format!("v{CACHE_FORMAT_VERSION}")));
        assert!(s.contains("7.3.0"));
        assert!(s.contains("CLASSIC-SOLID"));
        assert!(s.ends_with("house.svg"));
    }
}
