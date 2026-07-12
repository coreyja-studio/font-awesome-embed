//! Icon embedding tests. These run in two modes:
//!
//! - `--features test-icons` (CI): HTTP is faked with placeholder SVGs.
//! - No `test-icons`, with `FONT_AWESOME_TOKEN` set: every `fa!()` here
//!   hits the real Font Awesome GraphQL API at compile time, so this file
//!   doubles as live verification of the query shape and family/style
//!   pairings against release 7.3.0.
//!
//! Placeholder-specific assertions (`data-icon`/`data-style`) are gated on
//! the `test-icons` feature; everything else holds in both modes.

use font_awesome_embed::fa;

#[test]
fn solid_icon_returns_svg() {
    let svg = fa!("house", solid);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    #[cfg(feature = "test-icons")]
    {
        assert!(svg.contains(r#"data-icon="house""#));
        assert!(svg.contains(r#"data-style="solid""#));
    }
}

#[test]
fn brands_icon_returns_svg() {
    let svg = fa!("github", brands);
    assert!(svg.contains("<svg"));
    #[cfg(feature = "test-icons")]
    {
        assert!(svg.contains(r#"data-icon="github""#));
        assert!(svg.contains(r#"data-style="brands""#));
    }
}

#[test]
fn svg_has_accessibility_attributes() {
    let svg = fa!("user", regular);
    assert!(svg.contains(r#"fill="currentColor""#));
    assert!(svg.contains(r#"aria-hidden="true""#));
    assert!(svg.contains(r#"width="1em""#));
    assert!(svg.contains(r#"height="1em""#));
    assert!(svg.contains(r#"class="fa-svg""#));
}

// Every combination below exists in Font Awesome 7.3.0, so this test
// passes against the real API, not just the placeholder path.
#[test]
fn all_styles_compile() {
    let _ = fa!("star", solid);
    let _ = fa!("star", regular);
    let _ = fa!("star", light);
    let _ = fa!("star", thin);
    let _ = fa!("star", solid, family = sharp);
    let _ = fa!("star", regular, family = sharp);
    let _ = fa!("star", solid, family = duotone);
    let _ = fa!("star", solid, family = sharp_duotone);
    let _ = fa!("github", brands);
}

// One icon from each FA7 family, using the single style each family
// actually ships (e.g. pixel is regular-only, utility is semibold-only).
#[test]
fn fa7_families_compile() {
    let _ = fa!("star", regular, family = chisel);
    let _ = fa!("star", solid, family = etch);
    let _ = fa!("star", thin, family = graphite);
    let _ = fa!("star", regular, family = jelly);
    let _ = fa!("star", regular, family = jelly_duo);
    let _ = fa!("star", regular, family = jelly_fill);
    let _ = fa!("star", solid, family = mosaic);
    let _ = fa!("star", solid, family = notdog);
    let _ = fa!("star", solid, family = notdog_duo);
    let _ = fa!("star", regular, family = pixel);
    let _ = fa!("star", regular, family = slab);
    let _ = fa!("star", regular, family = slab_duo);
    let _ = fa!("star", regular, family = slab_press);
    let _ = fa!("star", regular, family = slab_press_duo);
    let _ = fa!("star", light, family = thumbprint);
    let _ = fa!("star", semibold, family = utility);
    let _ = fa!("star", semibold, family = utility_duo);
    let _ = fa!("star", semibold, family = utility_fill);
    let _ = fa!("star", solid, family = vellum);
    let _ = fa!("star", semibold, family = whiteboard);
}

#[test]
fn svg_is_static_str() {
    let svg: &'static str = fa!("check", solid);
    assert!(!svg.is_empty());
}
