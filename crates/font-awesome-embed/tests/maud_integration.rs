#![cfg(feature = "maud")]

use font_awesome_embed::maud::fa;

#[test]
fn maud_fa_returns_preescaped() {
    let result = fa!("house", solid);
    // PreEscaped<&str> implements Display, which renders the raw HTML
    let rendered = result.0;
    assert!(rendered.contains("<svg"));
    assert!(rendered.contains("</svg>"));
}

#[test]
fn maud_fa_in_html_template() {
    let markup = maud::html! {
        button {
            (fa!("github", brands))
            " GitHub"
        }
    };
    let html = markup.into_string();
    assert!(html.contains("<button>"));
    assert!(html.contains("<svg"));
    assert!(html.contains(" GitHub"));
}
