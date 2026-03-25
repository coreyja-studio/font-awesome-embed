use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::postprocess;

/// Get the cache directory for storing fetched SVGs.
fn cache_dir() -> PathBuf {
    // Prefer OUT_DIR (set by cargo during builds) for reproducible caching
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let dir = PathBuf::from(out_dir).join("fa-cache");
        let _ = fs::create_dir_all(&dir);
        return dir;
    }

    // Fall back to a temp directory
    let dir = std::env::temp_dir().join("font-awesome-embed-cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Check the filesystem cache for a previously fetched icon.
fn cache_get(name: &str, style: &str) -> Option<String> {
    let path = cache_dir().join(style).join(format!("{name}.svg"));
    fs::read_to_string(path).ok()
}

/// Store a fetched icon in the filesystem cache.
fn cache_set(name: &str, style: &str, svg: &str) {
    let dir = cache_dir().join(style);
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{name}.svg"));
    let _ = fs::write(path, svg);
}

#[derive(Deserialize)]
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
    release: ReleaseData,
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

/// Exchange the Font Awesome API token for an access token.
fn get_access_token(api_token: &str) -> Result<String, String> {
    let resp: TokenResponse = ureq::post("https://api.fontawesome.com/token")
        .header("Authorization", &format!("Bearer {api_token}"))
        .send_empty()
        .map_err(|e| format!("token exchange request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse token response: {e}"))?;

    Ok(resp.access_token)
}

/// Fetch an icon SVG from the Font Awesome GraphQL API.
fn fetch_from_api(name: &str, family: &str, style: &str) -> Result<String, String> {
    let api_token = std::env::var("FONT_AWESOME_TOKEN").map_err(|_| {
        "FONT_AWESOME_TOKEN environment variable not set. \
         Set it to your Font Awesome API token, or use the `test-icons` feature for development."
            .to_string()
    })?;

    let access_token = get_access_token(&api_token)?;

    let query = format!(
        r#"{{
            release(version: "latest") {{
                icon(id: "{name}") {{
                    svgs(filter: {{ familyStyle: {{ family: {family}, style: {style} }} }}) {{
                        html
                    }}
                }}
            }}
        }}"#
    );

    let body = serde_json::json!({ "query": query });

    let resp: GraphQLResponse = ureq::post("https://api.fontawesome.com")
        .header("Authorization", &format!("Bearer {access_token}"))
        .send_json(&body)
        .map_err(|e| format!("GraphQL request failed: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("failed to parse GraphQL response: {e}"))?;

    if let Some(errors) = resp.errors {
        let msgs: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
        return Err(format!("GraphQL errors: {}", msgs.join("; ")));
    }

    let data = resp.data.ok_or("no data in GraphQL response")?;
    let icon = data
        .release
        .icon
        .ok_or_else(|| format!("icon `{name}` not found"))?;
    let svg = icon
        .svgs
        .into_iter()
        .next()
        .ok_or_else(|| format!("no SVG found for icon `{name}` with style `{style}`"))?;

    Ok(svg.html)
}

/// Get an icon SVG, checking cache first, then fetching from the API.
pub fn get_icon(name: &str, family: &str, style: &str, cache_key: &str) -> Result<String, String> {
    // Check cache first
    if let Some(cached) = cache_get(name, cache_key) {
        return Ok(cached);
    }

    // Fetch from API
    let raw_svg = fetch_from_api(name, family, style)?;

    // Post-process the SVG
    let processed = postprocess::process_svg(&raw_svg);

    // Cache the result
    cache_set(name, cache_key, &processed);

    Ok(processed)
}
