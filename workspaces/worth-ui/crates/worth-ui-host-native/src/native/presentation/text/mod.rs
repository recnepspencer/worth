//! Native semantic-text presentation boundary.
//!
//! This boundary admits borrowed alpha raster output without retaining pixels or
//! performing atlas, GPU-upload, or glyph-run effects.

mod attribution;
mod commands;
mod sampled_geometry;
pub(crate) use attribution::mechanic_contains_run;
pub(in crate::native::presentation) use sampled_geometry::{sampled_glyph, sampled_image};

pub(super) use commands::glyph_vertices;
pub(crate) use commands::{
    clip_glyph_command, plan_glyph_commands, plan_raw_glyph_commands, source_is_intrinsic_color,
    UiNativeGlyphCommand, UiNativeGlyphCommandDenial,
};
