use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

mod fetch;
mod postprocess;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaFamily {
    Classic,
    Duotone,
    Sharp,
    SharpDuotone,
    Chisel,
    Etch,
    Graphite,
    Jelly,
    JellyDuo,
    JellyFill,
    Mosaic,
    Notdog,
    NotdogDuo,
    Pixel,
    Slab,
    SlabDuo,
    SlabPress,
    SlabPressDuo,
    Thumbprint,
    Utility,
    UtilityDuo,
    UtilityFill,
    Vellum,
    Whiteboard,
}

impl FaFamily {
    fn from_ident(ident: &Ident) -> syn::Result<Self> {
        Self::from_str(&ident.to_string()).map_err(|msg| syn::Error::new(ident.span(), msg))
    }

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "classic" => Ok(Self::Classic),
            "duotone" => Ok(Self::Duotone),
            "sharp" => Ok(Self::Sharp),
            "sharp_duotone" => Ok(Self::SharpDuotone),
            "chisel" => Ok(Self::Chisel),
            "etch" => Ok(Self::Etch),
            "graphite" => Ok(Self::Graphite),
            "jelly" => Ok(Self::Jelly),
            "jelly_duo" => Ok(Self::JellyDuo),
            "jelly_fill" => Ok(Self::JellyFill),
            "mosaic" => Ok(Self::Mosaic),
            "notdog" => Ok(Self::Notdog),
            "notdog_duo" => Ok(Self::NotdogDuo),
            "pixel" => Ok(Self::Pixel),
            "slab" => Ok(Self::Slab),
            "slab_duo" => Ok(Self::SlabDuo),
            "slab_press" => Ok(Self::SlabPress),
            "slab_press_duo" => Ok(Self::SlabPressDuo),
            "thumbprint" => Ok(Self::Thumbprint),
            "utility" => Ok(Self::Utility),
            "utility_duo" => Ok(Self::UtilityDuo),
            "utility_fill" => Ok(Self::UtilityFill),
            "vellum" => Ok(Self::Vellum),
            "whiteboard" => Ok(Self::Whiteboard),
            other => Err(format!(
                "unknown Font Awesome family: `{other}`. Expected one of: \
                 classic, duotone, sharp, sharp_duotone, chisel, etch, graphite, \
                 jelly, jelly_duo, jelly_fill, mosaic, notdog, notdog_duo, pixel, \
                 slab, slab_duo, slab_press, slab_press_duo, thumbprint, \
                 utility, utility_duo, utility_fill, vellum, whiteboard"
            )),
        }
    }
}

impl FaFamily {
    fn graphql_value(&self) -> &'static str {
        match self {
            Self::Classic => "CLASSIC",
            Self::Duotone => "DUOTONE",
            Self::Sharp => "SHARP",
            Self::SharpDuotone => "SHARP_DUOTONE",
            Self::Chisel => "CHISEL",
            Self::Etch => "ETCH",
            Self::Graphite => "GRAPHITE",
            Self::Jelly => "JELLY",
            Self::JellyDuo => "JELLY_DUO",
            Self::JellyFill => "JELLY_FILL",
            Self::Mosaic => "MOSAIC",
            Self::Notdog => "NOTDOG",
            Self::NotdogDuo => "NOTDOG_DUO",
            Self::Pixel => "PIXEL",
            Self::Slab => "SLAB",
            Self::SlabDuo => "SLAB_DUO",
            Self::SlabPress => "SLAB_PRESS",
            Self::SlabPressDuo => "SLAB_PRESS_DUO",
            Self::Thumbprint => "THUMBPRINT",
            Self::Utility => "UTILITY",
            Self::UtilityDuo => "UTILITY_DUO",
            Self::UtilityFill => "UTILITY_FILL",
            Self::Vellum => "VELLUM",
            Self::Whiteboard => "WHITEBOARD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaStyle {
    Brands,
    Duotone,
    Light,
    Regular,
    Semibold,
    Solid,
    Thin,
}

impl FaStyle {
    fn from_ident(ident: &Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "brands" => Ok(Self::Brands),
            "duotone" => Ok(Self::Duotone),
            "light" => Ok(Self::Light),
            "regular" => Ok(Self::Regular),
            "semibold" => Ok(Self::Semibold),
            "solid" => Ok(Self::Solid),
            "thin" => Ok(Self::Thin),
            other => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unknown Font Awesome style: `{other}`. Expected one of: \
                     brands, duotone, light, regular, semibold, solid, thin"
                ),
            )),
        }
    }
}

impl FaStyle {
    fn graphql_value(&self) -> &'static str {
        match self {
            Self::Brands => "BRANDS",
            Self::Duotone => "DUOTONE",
            Self::Light => "LIGHT",
            Self::Regular => "REGULAR",
            Self::Semibold => "SEMIBOLD",
            Self::Solid => "SOLID",
            Self::Thin => "THIN",
        }
    }
}

struct FaInput {
    name: String,
    name_span: proc_macro2::Span,
    style: FaStyle,
    family: FaFamily,
    class: Option<String>,
}

/// Validate a `class = "..."` value: it is spliced into the SVG's class
/// attribute, so anything that could break out of the attribute is rejected.
fn validate_class(class: &str) -> Result<(), String> {
    if class.is_empty() {
        return Err("class must not be empty".to_string());
    }
    if class.contains(['"', '<', '>']) || class.chars().any(char::is_control) {
        return Err(format!(
            "invalid class `{class}`: classes may not contain quotes, angle \
             brackets, or control characters"
        ));
    }
    Ok(())
}

impl Parse for FaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_lit: LitStr = input.parse()?;
        let name_span = name_lit.span();
        // Reject anything outside the FA slug charset at parse time — this
        // is the security boundary for the cache path and a friendlier
        // error than an API "not found" for typos like `fa!("House", ...)`.
        fetch::validate_icon_name(&name_lit.value())
            .map_err(|msg| syn::Error::new(name_span, msg))?;
        let _comma: Token![,] = input.parse()?;
        let style_ident: Ident = input.parse()?;
        let style = FaStyle::from_ident(&style_ident)?;

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
                family = Some(FaFamily::from_ident(&family_ident)?);
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
        Ok(val) => FaFamily::from_str(&val).map_err(|msg| {
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

    let svg = match fetch::get_icon(name, family_gql, style_gql) {
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
        Some(extra) => postprocess::inject_classes(&svg, extra),
        None => svg,
    };

    quote! { #svg }.into()
}
