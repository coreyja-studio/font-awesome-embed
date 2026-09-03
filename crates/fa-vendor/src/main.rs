use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;
use font_awesome_embed_core::{FaFamily, FaStyle, FetchMode, get_icon};
use serde::Deserialize;

/// Generate a typed TypeScript module of Font Awesome SVGs.
#[derive(Parser)]
struct Cli {
    /// Path to icons.toml manifest file
    manifest: PathBuf,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Use placeholder SVGs (no network/token needed)
    #[arg(long)]
    placeholders: bool,
}

#[derive(Deserialize)]
struct IconsManifest {
    icon: Vec<IconEntry>,
}

#[derive(Deserialize)]
struct IconEntry {
    name: String,
    style: String,
    #[serde(default = "default_family_str")]
    family: String,
}

/// Default family: `FA_DEFAULT_FAMILY` env var, or "classic" if unset.
///
/// Serde calls this for each entry that omits the `family` field, so an
/// omitted family and an explicit `family = "<the default>"` produce the
/// same string and the dedup check reports them as duplicates. (With the
/// env var unset, that default is "classic".)
///
/// The value is validated up front by `validate_default_family` so a typo
/// here is reported against the env var rather than against whichever
/// manifest entry happened to omit `family`.
fn default_family_str() -> String {
    std::env::var("FA_DEFAULT_FAMILY").unwrap_or_else(|_| "classic".to_string())
}

/// Reject an unparseable `FA_DEFAULT_FAMILY` with a message that names the
/// env var, mirroring the `fa!()` macro's behaviour.
fn validate_default_family(value: Option<&str>) -> Result<(), String> {
    match value {
        Some(val) => val
            .parse::<FaFamily>()
            .map(|_| ())
            .map_err(|msg| format!("invalid FA_DEFAULT_FAMILY env var: {msg}")),
        None => Ok(()),
    }
}

struct TsIconEntry {
    const_name: String,
    svg: String,
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let manifest_content = std::fs::read_to_string(&cli.manifest)
        .map_err(|e| format!("failed to read manifest {}: {e}", cli.manifest.display()))?;

    let manifest: IconsManifest =
        toml::from_str(&manifest_content).map_err(|e| format!("failed to parse manifest: {e}"))?;

    let mode = if cli.placeholders {
        FetchMode::Placeholder
    } else {
        FetchMode::Real
    };

    let ts = generate(&manifest, mode)?;

    match &cli.output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("failed to create output directory: {e}"))?;
            }
            std::fs::write(path, &ts).map_err(|e| format!("failed to write output: {e}"))?;
        }
        None => {
            print!("{ts}");
        }
    }

    Ok(())
}

fn generate(manifest: &IconsManifest, mode: FetchMode) -> Result<String, String> {
    validate_default_family(std::env::var("FA_DEFAULT_FAMILY").ok().as_deref())?;

    let mut entries = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for icon in &manifest.icon {
        let family = icon.family.parse::<FaFamily>()?;
        let style = icon.style.parse::<FaStyle>()?;

        let const_name = ts_const_name(&icon.name, &icon.style, &icon.family);
        if !seen.insert(const_name.clone()) {
            return Err(format!(
                "duplicate icon entry: const name `{const_name}` is already used \
                 (name=`{}`, style=`{}`, family=`{}`)",
                icon.name, icon.style, icon.family
            ));
        }

        let svg = get_icon(
            &icon.name,
            family.graphql_value(),
            style.graphql_value(),
            mode,
        )?;

        entries.push(TsIconEntry { const_name, svg });
    }

    Ok(generate_ts_module(&entries))
}

/// Build a TS-safe const name from icon name, style, and family.
///
/// Always prefixed with `fa_` to ensure valid JavaScript identifiers —
/// Font Awesome ships icons whose slugs start with a digit (e.g. `500px`,
/// `42-group`, `1`, `00`), and `validate_icon_name` permits leading digits.
/// Without the prefix, `500px` → `500px_solid` would be a SyntaxError.
///
/// Hyphens → underscores. Family appended only when not "classic".
fn ts_const_name(name: &str, style: &str, family: &str) -> String {
    let base = name.replace('-', "_");
    let style_lower = style.to_lowercase();
    if family == "classic" {
        format!("fa_{base}_{style_lower}")
    } else {
        format!("fa_{base}_{style_lower}_{}", family.replace('-', "_"))
    }
}

/// Escape a string for safe embedding in a TS template literal.
///
/// Escapes backslashes (first, to avoid double-escaping), backticks,
/// `${` (template expression), and `</` (prevents `</script>` breakout
/// if the module is ever inlined in a `<script>` tag — HTML parsers treat
/// tag names case-insensitively, so replacing `</` covers all variants).
fn escape_ts(s: &str) -> String {
    s.replace('\\', r"\\")
        .replace('`', r"\`")
        .replace("${", r"\${")
        .replace("</", r"<\/")
}

/// Generate the full TS module from icon entries.
fn generate_ts_module(entries: &[TsIconEntry]) -> String {
    let mut out = String::new();

    out.push_str("// AUTO-GENERATED by fa-vendor — do not edit manually.\n");
    out.push_str(
        "// Icons are fetched from Font Awesome under your license; do not commit this file.\n\n",
    );

    for entry in entries {
        let escaped = escape_ts(&entry.svg);
        out.push_str(&format!(
            "export const {} = `{}`;\n",
            entry.const_name, escaped
        ));
    }

    if entries.is_empty() {
        out.push_str("export type FaIconName = never;\n");
    } else {
        out.push_str("\nexport type FaIconName =");
        for (i, entry) in entries.iter().enumerate() {
            if i == 0 {
                out.push_str(&format!(" \"{}\"", entry.const_name));
            } else {
                out.push_str(&format!(" | \"{}\"", entry.const_name));
            }
        }
        out.push_str(";\n");
    }

    out.push_str("\nexport const faIcons: Record<FaIconName, string> = {\n");
    for entry in entries {
        out.push_str(&format!("  {},\n", entry.const_name));
    }
    out.push_str("};\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn const_name_classic() {
        assert_eq!(ts_const_name("house", "solid", "classic"), "fa_house_solid");
    }

    #[test]
    fn const_name_hyphenated() {
        assert_eq!(
            ts_const_name("arrow-up-right-from-square", "regular", "classic"),
            "fa_arrow_up_right_from_square_regular"
        );
    }

    #[test]
    fn const_name_non_classic_family() {
        assert_eq!(
            ts_const_name("star", "solid", "notdog"),
            "fa_star_solid_notdog"
        );
    }

    #[test]
    fn const_name_digit_leading() {
        assert_eq!(
            ts_const_name("500px", "brands", "classic"),
            "fa_500px_brands"
        );
        assert_eq!(
            ts_const_name("42-group", "solid", "classic"),
            "fa_42_group_solid"
        );
    }

    #[test]
    fn escape_ts_handles_backticks() {
        assert_eq!(escape_ts("hello`world"), "hello\\`world");
    }

    #[test]
    fn escape_ts_handles_template_expr() {
        assert_eq!(escape_ts("${evil}"), "\\${evil}");
    }

    #[test]
    fn escape_ts_handles_closing_tag() {
        assert_eq!(escape_ts("</script>"), "<\\/script>");
        assert_eq!(escape_ts("</svg>"), "<\\/svg>");
        assert_eq!(escape_ts("</ScRiPt>"), "<\\/ScRiPt>");
    }

    #[test]
    fn generate_ts_module_structure() {
        let entries = vec![
            TsIconEntry {
                const_name: "fa_house_solid".to_string(),
                svg: "<svg>test</svg>".to_string(),
            },
            TsIconEntry {
                const_name: "fa_github_brands".to_string(),
                svg: "<svg>test2</svg>".to_string(),
            },
        ];
        let ts = generate_ts_module(&entries);

        assert!(ts.contains("export const fa_house_solid = `<svg>test<\\/svg>`;"));
        assert!(ts.contains("export const fa_github_brands = `<svg>test2<\\/svg>`;"));
        assert!(ts.contains(r#"export type FaIconName = "fa_house_solid" | "fa_github_brands";"#));
        assert!(ts.contains("export const faIcons: Record<FaIconName, string> = {"));
        assert!(ts.contains("  fa_house_solid,"));
        assert!(ts.contains("  fa_github_brands,"));
    }

    #[test]
    fn generate_empty_manifest() {
        let ts = generate_ts_module(&[]);
        assert!(ts.contains("export type FaIconName = never;"));
    }

    #[test]
    fn generate_placeholder_mode_offline() {
        let toml_str = r#"
[[icon]]
name = "house"
style = "solid"

[[icon]]
name = "github"
style = "brands"

[[icon]]
name = "star"
style = "solid"
family = "notdog"

[[icon]]
name = "500px"
style = "brands"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let ts = generate(&manifest, FetchMode::Placeholder).unwrap();

        assert!(ts.contains("export const fa_house_solid ="));
        assert!(ts.contains("export const fa_github_brands ="));
        assert!(ts.contains("export const fa_star_solid_notdog ="));
        assert!(ts.contains("export const fa_500px_brands ="));
        assert!(ts.contains(
            r#""fa_house_solid" | "fa_github_brands" | "fa_star_solid_notdog" | "fa_500px_brands""#
        ));
        assert!(ts.contains(r#"data-icon="house""#));
    }

    #[test]
    fn duplicate_entries_error() {
        let toml_str = r#"
[[icon]]
name = "house"
style = "solid"

[[icon]]
name = "house"
style = "solid"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let result = generate(&manifest, FetchMode::Placeholder);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn duplicate_explicit_vs_default_family_error() {
        let toml_str = r#"
[[icon]]
name = "house"
style = "solid"

[[icon]]
name = "house"
style = "solid"
family = "classic"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let result = generate(&manifest, FetchMode::Placeholder);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("duplicate"));
    }

    #[test]
    fn duplicate_const_name_collision() {
        // Two distinct (name, style, family) triples that produce the same
        // const name because `duotone` is both a valid style and a valid
        // family, and hyphens in icon names are converted to underscores.
        //   ts_const_name("arrow", "solid", "duotone")     -> fa_arrow_solid_duotone
        //   ts_const_name("arrow-solid", "duotone", "classic") -> fa_arrow_solid_duotone
        let toml_str = r#"
[[icon]]
name = "arrow"
style = "solid"
family = "duotone"

[[icon]]
name = "arrow-solid"
style = "duotone"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let result = generate(&manifest, FetchMode::Placeholder);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("duplicate"));
        assert!(err.contains("fa_arrow_solid_duotone"));
    }

    #[test]
    fn default_family_env_validated_by_name() {
        assert!(validate_default_family(None).is_ok());
        assert!(validate_default_family(Some("sharp")).is_ok());
        let err = validate_default_family(Some("comic_sans")).unwrap_err();
        assert!(err.contains("FA_DEFAULT_FAMILY"), "unexpected error: {err}");
    }

    #[test]
    fn invalid_family_error() {
        let toml_str = r#"
[[icon]]
name = "house"
style = "solid"
family = "comic_sans"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let result = generate(&manifest, FetchMode::Placeholder);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown Font Awesome family"));
    }

    #[test]
    fn invalid_style_error() {
        let toml_str = r#"
[[icon]]
name = "house"
style = "italic"
"#;
        let manifest: IconsManifest = toml::from_str(toml_str).unwrap();
        let result = generate(&manifest, FetchMode::Placeholder);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown Font Awesome style"));
    }
}
