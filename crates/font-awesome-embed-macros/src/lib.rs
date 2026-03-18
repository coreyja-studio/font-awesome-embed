use proc_macro::TokenStream;
#[cfg(feature = "test-icons")]
use quote::quote;
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

struct FaInput {
    #[cfg_attr(not(feature = "test-icons"), allow(dead_code))]
    name: LitStr,
    style: Ident,
}

impl Parse for FaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let style: Ident = input.parse()?;
        Ok(FaInput { name, style })
    }
}

const VALID_STYLES: &[&str] = &[
    "solid",
    "regular",
    "brands",
    "light",
    "thin",
    "duotone",
    "sharp_solid",
    "sharp_regular",
];

#[proc_macro]
pub fn fa(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FaInput);

    let style_str = input.style.to_string();
    if !VALID_STYLES.contains(&style_str.as_str()) {
        let valid = VALID_STYLES.join(", ");
        return syn::Error::new_spanned(
            input.style,
            format!("unknown Font Awesome style `{style_str}`. Expected one of: {valid}"),
        )
        .to_compile_error()
        .into();
    }

    #[cfg(feature = "test-icons")]
    {
        let name = input.name.value();
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="1em" height="1em" fill="currentColor" aria-hidden="true" class="fa-svg" data-icon="{name}" data-style="{style_str}"><rect width="24" height="24" rx="4" opacity="0.2"/><text x="12" y="17" font-size="10" text-anchor="middle" fill="currentColor">{initial}</text></svg>"#,
            name = name,
            style_str = style_str,
            initial = name.chars().next().unwrap_or('?'),
        );
        quote! { #svg }.into()
    }

    #[cfg(not(feature = "test-icons"))]
    {
        let _ = &input;
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Font Awesome icon fetching is not yet implemented. \
             Use the `test-icons` feature for development: \
             font-awesome-embed = { version = \"0.1\", features = [\"test-icons\"] }",
        )
        .to_compile_error()
        .into()
    }
}
