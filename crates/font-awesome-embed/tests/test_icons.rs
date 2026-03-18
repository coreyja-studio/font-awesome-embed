#[cfg(feature = "test-icons")]
mod test_icons {
    use font_awesome_embed::fa;

    #[test]
    fn returns_svg_string() {
        let svg: &str = fa!("house", solid);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn includes_icon_name() {
        let svg: &str = fa!("github", brands);
        assert!(svg.contains(r#"data-icon="github""#));
    }

    #[test]
    fn includes_style() {
        let svg: &str = fa!("star", regular);
        assert!(svg.contains(r#"data-style="regular""#));
    }

    #[test]
    fn includes_accessibility_attrs() {
        let svg: &str = fa!("check", solid);
        assert!(svg.contains(r#"aria-hidden="true""#));
        assert!(svg.contains(r#"fill="currentColor""#));
    }

    #[test]
    fn includes_sizing() {
        let svg: &str = fa!("user", solid);
        assert!(svg.contains(r#"width="1em""#));
        assert!(svg.contains(r#"height="1em""#));
    }

    #[test]
    fn includes_css_class() {
        let svg: &str = fa!("bell", solid);
        assert!(svg.contains(r#"class="fa-svg""#));
    }
}
