use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

use font_awesome_embed_core::{
    FaFamily, FaStyle, FetchMode, get_icon, inject_classes, validate_class, validate_icon_name,
};

// from_ident wrappers — call core's FromStr and wrap in syn::Error.
// These are free functions (not inherent impls on FaFamily/FaStyle)
// because orphan rules prevent adding inherent impls to types defined
// in another crate.
fn family_from_ident(ident: &Ident) -> syn::Result<FaFamily> {
    ident
        .to_string()
        .parse::<FaFamily>()
        .map_err(|msg| syn::Error::new(ident.span(), msg))
}

fn style_from_ident(ident: &Ident) -> syn::Result<FaStyle> {
    ident
        .to_string()
        .parse::<FaStyle>()
        .map_err(|msg| syn::Error::new(ident.span(), msg))
}

struct FaInput {
    name: String,
    name_span: proc_macro2::Span,
    style: FaStyle,
    family: FaFamily,
    class: Option<String>,
}

impl Parse for FaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_lit: LitStr = input.parse()?;
        let name_span = name_lit.span();
        // Reject anything outside the FA slug charset at parse time — this
        // is the security boundary for the cache path and a friendlier
        // error than an API "not found" for typos like `fa!("House", ...)`.
        validate_icon_name(&name_lit.value()).map_err(|msg| syn::Error::new(name_span, msg))?;
        let _comma: Token![,] = input.parse()?;
        let style_ident: Ident = input.parse()?;
        let style = style_from_ident(&style_ident)?;

        let mut family: Option<FaFamily> = None;
        let mut class: Option<String> = None;

        while input.peek(Token![,]) && input.peek2(Ident) {
            let _comma: Token![,] = input.parse()?;
            let key: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            if key == "family" {
                if family.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate `family` argument"));
                }
                let family_ident: Ident = input.parse()?;
                family = Some(family_from_ident(&family_ident)?);
            } else if key == "class" {
                if class.is_some() {
                    return Err(syn::Error::new(key.span(), "duplicate `class` argument"));
                }
                let class_lit: LitStr = input.parse()?;
                let value = class_lit.value();
                validate_class(&value).map_err(|msg| syn::Error::new(class_lit.span(), msg))?;
                class = Some(value);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    format!(
                        "unexpected argument `{key}`. Expected `family = <family>` \
                         or `class = \"...\"`"
                    ),
                ));
            }
        }

        // Allow trailing comma
        let _: Option<Token![,]> = input.parse()?;

        let family = match family {
            Some(family) => family,
            None => default_family()?,
        };

        Ok(FaInput {
            name: name_lit.value(),
            name_span,
            style,
            family,
            class,
        })
    }
}

fn default_family() -> syn::Result<FaFamily> {
    match std::env::var("FA_DEFAULT_FAMILY") {
        Ok(val) => val.parse::<FaFamily>().map_err(|msg| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("invalid FA_DEFAULT_FAMILY env var: {msg}"),
            )
        }),
        Err(_) => Ok(FaFamily::Classic),
    }
}

#[proc_macro]
#[allow(clippy::needless_return)]
pub fn fa(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FaInput);
    let name = &input.name;
    let family_gql = input.family.graphql_value();
    let style_gql = input.style.graphql_value();

    #[cfg(feature = "test-icons")]
    let mode = FetchMode::Placeholder;
    #[cfg(not(feature = "test-icons"))]
    let mode = FetchMode::Real;

    let svg = match get_icon(name, family_gql, style_gql, mode) {
        Ok(svg) => svg,
        Err(e) => {
            let msg = format!(
                "failed to fetch Font Awesome icon `{name}` ({}/{}): {e}",
                input.family.graphql_value(),
                input.style.graphql_value()
            );
            return syn::Error::new(input.name_span, msg)
                .to_compile_error()
                .into();
        }
    };

    let svg = match &input.class {
        Some(extra) => inject_classes(&svg, extra),
        None => svg,
    };

    quote! { #svg }.into()
}
