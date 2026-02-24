use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaStyle {
    Solid,
    Regular,
    Brands,
    Light,
    Thin,
    Duotone,
    SharpSolid,
    SharpRegular,
    Kit,
    KitDuotone,
}

impl FaStyle {
    fn from_ident(ident: &Ident) -> syn::Result<Self> {
        match ident.to_string().as_str() {
            "solid" => Ok(Self::Solid),
            "regular" => Ok(Self::Regular),
            "brands" => Ok(Self::Brands),
            "light" => Ok(Self::Light),
            "thin" => Ok(Self::Thin),
            "duotone" => Ok(Self::Duotone),
            "sharp_solid" => Ok(Self::SharpSolid),
            "sharp_regular" => Ok(Self::SharpRegular),
            "kit" => Ok(Self::Kit),
            "kit_duotone" => Ok(Self::KitDuotone),
            other => Err(syn::Error::new(
                ident.span(),
                format!(
                    "unknown Font Awesome style: `{other}`. Expected one of: \
                     solid, regular, brands, light, thin, duotone, sharp_solid, sharp_regular, \
                     kit, kit_duotone"
                ),
            )),
        }
    }
}

#[allow(dead_code)]
impl FaStyle {
    fn family(&self) -> &'static str {
        match self {
            Self::Solid | Self::Regular | Self::Brands | Self::Light | Self::Thin => "CLASSIC",
            Self::Duotone => "DUOTONE",
            Self::SharpSolid | Self::SharpRegular => "SHARP",
            Self::Kit => "KIT",
            Self::KitDuotone => "KIT_DUOTONE",
        }
    }

    fn style(&self) -> &'static str {
        match self {
            Self::Solid | Self::Duotone | Self::SharpSolid => "SOLID",
            Self::Regular | Self::SharpRegular => "REGULAR",
            Self::Brands => "BRANDS",
            Self::Light => "LIGHT",
            Self::Thin => "THIN",
            Self::Kit | Self::KitDuotone => "CUSTOM",
        }
    }
}

#[allow(dead_code)]
struct FaInput {
    name: String,
    style: FaStyle,
}

impl Parse for FaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_lit: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let style_ident: Ident = input.parse()?;
        let style = FaStyle::from_ident(&style_ident)?;
        let _: Option<Token![,]> = input.parse()?;
        Ok(FaInput {
            name: name_lit.value(),
            style,
        })
    }
}

#[proc_macro]
pub fn fa(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as FaInput);
    quote! { "" }.into()
}
