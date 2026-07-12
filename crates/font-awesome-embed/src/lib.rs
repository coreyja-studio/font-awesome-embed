pub use font_awesome_embed_macros::fa;

#[cfg(feature = "maud")]
pub mod maud {
    /// `fa!` wrapped in `maud::PreEscaped` for direct use in `html! {}`
    /// splices. Forwards any named arguments (`family = ...`, `class = ...`)
    /// to the underlying proc macro.
    #[macro_export]
    macro_rules! fa_maud {
        ($name:literal, $style:ident $(, $key:ident = $value:tt)* $(,)?) => {
            ::maud::PreEscaped($crate::fa!($name, $style $(, $key = $value)*))
        };
    }

    pub use crate::fa_maud as fa;
}
