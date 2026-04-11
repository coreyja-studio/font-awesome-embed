/// Get an icon SVG, checking cache first, then fetching from the API.
///
/// When the `test-icons` feature is enabled, returns a placeholder SVG
/// without making any network calls.
pub fn get_icon(name: &str, family: &str, style: &str, cache_key: &str) -> Result<String, String> {
    // In test-icons mode, return a placeholder SVG without any network calls
    #[cfg(feature = "test-icons")]
    {
        let _ = (family, cache_key); // suppress unused warnings
        #[allow(clippy::needless_return)]
        return Ok(format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" fill="currentColor" aria-hidden="true" width="1em" height="1em" class="fa-svg" data-icon="{name}" data-style="{style}">{}</svg>"#,
            r#"<rect width="512" height="512" fill="currentColor" opacity="0.2"/>"#
        ));
    }

    // Real mode: fetch from API with caching
    #[cfg(not(feature = "test-icons"))]
    {
        use crate::postprocess;

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
}

// --- Everything below is only compiled when test-icons is NOT enabled ---

#[cfg(not(feature = "test-icons"))]
use std::fs;
#[cfg(not(feature = "test-icons"))]
use std::path::PathBuf;

#[cfg(not(feature = "test-icons"))]
use serde::Deserialize;

#[cfg(not(feature = "test-icons"))]
fn cache_dir() -> PathBuf {
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let dir = PathBuf::from(out_dir).join("fa-cache");
        let _ = fs::create_dir_all(&dir);
        return dir;
    }
    let dir = std::env::temp_dir().join("font-awesome-embed-cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

#[cfg(not(feature = "test-icons"))]
fn cache_get(name: &str, style: &str) -> Option<String> {
    let path = cache_dir().join(style).join(format!("{name}.svg"));
    fs::read_to_string(path).ok()
}

#[cfg(not(feature = "test-icons"))]
fn cache_set(name: &str, style: &str, svg: &str) {
    let dir = cache_dir().join(style);
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{name}.svg"));
    let _ = fs::write(path, svg);
}

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
    release: ReleaseData,
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

#[cfg(not(feature = "test-icons"))]
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

#[cfg(not(feature = "test-icons"))]
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
