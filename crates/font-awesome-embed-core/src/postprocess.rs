/// Post-process a raw Font Awesome SVG for embedding.
///
/// Applies:
/// - `fill="currentColor"` to inherit text color
/// - `aria-hidden="true"` for accessibility
/// - `width="1em" height="1em"` to scale with text
/// - `class="fa-svg"` as a global CSS hook
/// - Strips license/comment blocks to save binary size
pub fn process_svg(svg: &str) -> String {
    let mut result = svg.to_string();

    // Strip HTML/XML comments (license blocks, etc.)
    while let Some(start) = result.find("<!--") {
        if let Some(end) = result[start..].find("-->") {
            result.replace_range(start..start + end + 3, "");
        } else {
            break;
        }
    }

    // Inject attributes into the opening <svg> tag
    if let Some(svg_start) = result.find("<svg")
        && let Some(tag_end) = result[svg_start..].find('>')
    {
        let tag_end = svg_start + tag_end;
        let tag = &result[svg_start..tag_end];

        // Add fill="currentColor" if not present
        let mut attrs_to_add = String::new();
        if !tag.contains("fill=") {
            attrs_to_add.push_str(r#" fill="currentColor""#);
        }

        // Add aria-hidden="true" if not present
        if !tag.contains("aria-hidden") {
            attrs_to_add.push_str(r#" aria-hidden="true""#);
        }

        // Add width/height if not present
        if !tag.contains("width=") {
            attrs_to_add.push_str(r#" width="1em""#);
        }
        if !tag.contains("height=") {
            attrs_to_add.push_str(r#" height="1em""#);
        }

        // Add class if not present
        if !tag.contains("class=") {
            attrs_to_add.push_str(r#" class="fa-svg""#);
        }

        if !attrs_to_add.is_empty() {
            result.insert_str(svg_start + 4, &attrs_to_add);
        }
    }

    // Normalize whitespace — collapse runs of whitespace to single spaces
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_whitespace = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_whitespace {
                collapsed.push(' ');
            }
            prev_whitespace = true;
        } else {
            collapsed.push(ch);
            prev_whitespace = false;
        }
    }

    collapsed.trim().to_string()
}

/// Append caller-supplied classes to the `fa-svg` class the post-processor
/// guarantees on every cached SVG. Runs at macro expansion, after the cache,
/// so one cached icon serves every distinct `class = "..."` call site.
pub fn inject_classes(svg: &str, extra: &str) -> String {
    svg.replacen(
        r#"class="fa-svg""#,
        &format!(r#"class="fa-svg {extra}""#),
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_missing_attributes() {
        let input = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><path d="M0 0"/></svg>"#;
        let result = process_svg(input);

        assert!(result.contains(r#"fill="currentColor""#));
        assert!(result.contains(r#"aria-hidden="true""#));
        assert!(result.contains(r#"width="1em""#));
        assert!(result.contains(r#"height="1em""#));
        assert!(result.contains(r#"class="fa-svg""#));
    }

    #[test]
    fn preserves_existing_fill() {
        let input = r#"<svg fill="red" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><path d="M0 0"/></svg>"#;
        let result = process_svg(input);

        // Should not duplicate fill
        assert!(result.contains(r#"fill="red""#));
        assert!(!result.contains(r#"fill="currentColor""#));
    }

    #[test]
    fn injects_extra_classes() {
        let svg = process_svg(r#"<svg xmlns="x" viewBox="0 0 512 512"><path d="M0 0"/></svg>"#);
        let out = inject_classes(&svg, "text-xl text-red-400");
        assert!(out.contains(r#"class="fa-svg text-xl text-red-400""#));
        assert_eq!(out.matches("text-xl").count(), 1);
    }

    #[test]
    fn strips_comments() {
        let input = r#"<!-- License: MIT -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!-- icon --><path d="M0 0"/></svg>"#;
        let result = process_svg(input);

        assert!(!result.contains("<!--"));
        assert!(!result.contains("-->"));
        assert!(!result.contains("License"));
    }

    #[test]
    fn collapses_whitespace() {
        let input = "<svg xmlns=\"http://www.w3.org/2000/svg\"   viewBox=\"0 0 512 512\">\n  <path d=\"M0 0\"/>\n</svg>";
        let result = process_svg(input);

        assert!(!result.contains('\n'));
        assert!(!result.contains("  "));
    }
}
