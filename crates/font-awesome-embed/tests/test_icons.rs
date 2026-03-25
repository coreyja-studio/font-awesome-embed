use font_awesome_embed::fa;

#[test]
fn solid_icon_returns_svg() {
    let svg = fa!("house", solid);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains(r#"data-icon="house""#));
    assert!(svg.contains(r#"data-style="solid""#));
}

#[test]
fn brands_icon_returns_svg() {
    let svg = fa!("github", brands);
    assert!(svg.contains("<svg"));
    assert!(svg.contains(r#"data-icon="github""#));
    assert!(svg.contains(r#"data-style="brands""#));
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

#[test]
fn all_styles_compile() {
    let _ = fa!("star", solid);
    let _ = fa!("star", regular);
    let _ = fa!("star", light);
    let _ = fa!("star", thin);
    let _ = fa!("star", duotone);
    let _ = fa!("star", solid, family = sharp);
    let _ = fa!("star", regular, family = sharp);
    let _ = fa!("github", brands);
}

#[test]
fn svg_is_static_str() {
    let svg: &'static str = fa!("check", solid);
    assert!(!svg.is_empty());
}
