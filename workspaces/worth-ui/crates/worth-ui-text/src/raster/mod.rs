//! Phase 5 raster authority contracts owned by text mechanics.
//!
//! Text-owned qualified glyph demand, alpha-outline, and intrinsic-color raster.
//!
//! This module stops before atlas effects, GPU work, and presentation
//! settlement.

mod alpha;
mod alpha_admission;
mod alpha_record;
mod alpha_transaction_admission;
mod alpha_transaction_completion;
mod batch;
mod cache;
mod capacity;
mod color;
mod cost;
mod demand;
mod demand_candidate;
mod demand_geometry;
mod demand_identity;
mod denial;
mod key;
mod placement;
mod planning_geometry;
mod qualified_raster_admission;
mod source;

pub use alpha::{
    rasterize_alpha_outline, rasterize_alpha_outline_selection,
    rasterize_alpha_outline_selection_cached, rasterize_alpha_outline_transaction,
    UiAlphaRasterTransaction, UiAlphaRasterization,
};
pub use alpha_admission::{admit_alpha_outline, UiAlphaRasterAdmission};
pub use alpha_transaction_admission::{
    admit_alpha_outline_transaction, UiAlphaRasterTransactionAdmission,
};
pub use alpha_transaction_completion::{
    UiAlphaRasterBatchCompletion, UiAlphaRasterTransactionCompletion,
};
pub use batch::{
    UiAlphaRasterBatch, UiColorRasterBatch, UiGlyphRasterAdmissionDenial, UiGlyphRasterBatch,
    UiGlyphRasterRecord,
};
pub use cache::UiGlyphRasterCache;
pub use color::admission::{
    admit_intrinsic_color, admit_intrinsic_color_transaction, UiColorRasterAdmission,
    UiColorRasterTransactionAdmission,
};
pub use color::completion::{UiColorRasterBatchCompletion, UiColorRasterTransactionCompletion};
pub use color::{
    rasterize_intrinsic_color, rasterize_intrinsic_color_selection,
    rasterize_intrinsic_color_selection_cached, rasterize_intrinsic_color_transaction,
    UiColorRasterTransaction, UiColorRasterization,
};
pub use cost::{UiGlyphRasterCost, UiGlyphRasterLaneCost};
pub use demand::{
    derive_glyph_raster_demand, UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial,
    UiGlyphRasterDemandRequest, UiGlyphRasterDemandSelection, UiGlyphRasterScale,
};
pub use denial::UiGlyphRasterizationDenial;
pub use key::admit_raster_key;
pub use placement::UiGlyphRasterPlacement;
pub use source::{UiAlphaRasterKind, UiColorRasterKind, UiGlyphRasterFormat};
pub use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterBearing, UiGlyphRasterContentDigest,
    UiGlyphRasterExtent, UiGlyphRasterKey, UiGlyphRasterLane, UiGlyphRasterSource,
};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod demand_alpha_tests;
#[cfg(test)]
pub(crate) mod demand_identity_tests;
#[cfg(test)]
mod demand_ligature_tests;
#[cfg(test)]
mod demand_scope_tests;
#[cfg(test)]
mod demand_work_tests;

#[cfg(test)]
mod alpha_transaction_tests;
