pub use font_awesome_embed_macros::fa;

#[cfg(feature = "maud")]
pub mod maud {
    #[macro_export]
    macro_rules! fa_maud {
        ($name:literal, $style:ident) => {
            ::maud::PreEscaped($crate::fa!($name, $style))
        };
        ($name:literal, $style:ident, family = $family:ident) => {
            ::maud::PreEscaped($crate::fa!($name, $style, family = $family))
        };
    }

    pub use crate::fa_maud as fa;
}
