pub mod fetch;
pub mod postprocess;

// Re-export key items for convenience
pub use fetch::{FetchMode, get_icon, validate_icon_name};
pub use postprocess::{inject_classes, process_svg};

// --- Types moved from font-awesome-embed-macros/src/lib.rs ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaFamily {
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

impl std::str::FromStr for FaFamily {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
    pub fn graphql_value(&self) -> &'static str {
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
pub enum FaStyle {
    Brands,
    Duotone,
    Light,
    Regular,
    Semibold,
    Solid,
    Thin,
}

impl std::str::FromStr for FaStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "brands" => Ok(Self::Brands),
            "duotone" => Ok(Self::Duotone),
            "light" => Ok(Self::Light),
            "regular" => Ok(Self::Regular),
            "semibold" => Ok(Self::Semibold),
            "solid" => Ok(Self::Solid),
            "thin" => Ok(Self::Thin),
            other => Err(format!(
                "unknown Font Awesome style: `{other}`. Expected one of: \
                 brands, duotone, light, regular, semibold, solid, thin"
            )),
        }
    }
}

impl FaStyle {
    pub fn graphql_value(&self) -> &'static str {
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

/// Validate a `class = "..."` value (moved from macros/src/lib.rs).
/// Spliced into the SVG's class attribute, so anything that could break
/// out of the attribute is rejected.
pub fn validate_class(class: &str) -> Result<(), String> {
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
