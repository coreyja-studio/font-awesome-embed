use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

/// Get an icon SVG, checking cache first, then fetching from the API.
///
/// When the `test-icons` feature is enabled, the HTTP calls return fake
/// responses — but caching, query building, response parsing, and
/// post-processing all run the same code path as production.
pub fn get_icon(name: &str, family: &str, style: &str, cache_key: &str) -> Result<String, String> {
    use crate::postprocess;

    // Check cache first
    if let Some(cached) = cache_get(name, cache_key) {
        return Ok(cached);
    }

    // Fetch from API (HTTP calls are faked in test-icons mode)
    let raw_svg = fetch_from_api(name, family, style)?;

    // Post-process the SVG
    let processed = postprocess::process_svg(&raw_svg);

    // Cache the result
    cache_set(name, cache_key, &processed);

    Ok(processed)
}

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

fn cache_get(name: &str, style: &str) -> Option<String> {
    let path = cache_dir().join(style).join(format!("{name}.svg"));
    fs::read_to_string(path).ok()
}

fn cache_set(name: &str, style: &str, svg: &str) {
    let dir = cache_dir().join(style);
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{name}.svg"));
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

// --- HTTP transport (faked in test-icons mode) ---

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
        let resp: TokenResponse = ureq::post("https://api.fontawesome.com/token")
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
                release: ReleaseData {
                    icon: Some(IconData {
                        svgs: vec![SvgEntry {
                            html: format!(
                                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" data-icon="{name}" data-style="{style_lower}"><rect width="512" height="512" opacity="0.2"/></svg>"#
                            ),
                        }],
                    }),
                },
            }),
            errors: None,
        })
    }

    #[cfg(not(feature = "test-icons"))]
    {
        let _ = (name, style);
        let resp: GraphQLResponse = ureq::post("https://api.fontawesome.com")
            .header("Authorization", &format!("Bearer {access_token}"))
            .send_json(body)
            .map_err(|e| format!("GraphQL request failed: {e}"))?
            .body_mut()
            .read_json()
            .map_err(|e| format!("failed to parse GraphQL response: {e}"))?;

        Ok(resp)
    }
}

/// Fetch an icon SVG from the Font Awesome GraphQL API.
///
/// Builds the query, authenticates, sends the request, and extracts
/// the SVG from the response. In test-icons mode, the HTTP calls are
/// faked but query building and response parsing still run.
fn fetch_from_api(name: &str, family: &str, style: &str) -> Result<String, String> {
    let api_token = std::env::var("FONT_AWESOME_TOKEN").or_else(|_| {
        #[cfg(feature = "test-icons")]
        return Ok::<String, String>("test-token".to_string());

        #[cfg(not(feature = "test-icons"))]
        Err(
            "FONT_AWESOME_TOKEN environment variable not set. \
             Set it to your Font Awesome API token, or use the `test-icons` feature for development."
                .to_string(),
        )
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

    let resp = send_graphql_request(&access_token, &body, name, style)?;

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
