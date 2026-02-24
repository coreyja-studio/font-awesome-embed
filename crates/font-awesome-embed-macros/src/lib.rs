use proc_macro::TokenStream;
use syn::{Ident, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

struct FaInput {
    _name: LitStr,
    _style: Ident,
}

impl Parse for FaInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let style: Ident = input.parse()?;
        Ok(FaInput {
            _name: name,
            _style: style,
        })
    }
}

#[proc_macro]
pub fn fa(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as FaInput);
    todo!("not yet implemented")
}
