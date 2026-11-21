//! Opt-in custody of the original completed Native disclosed projection.
//! No body, DTO, authority, snapshot or comparison result is synthesized.

pub mod body;
pub mod comparison;
pub use body::OriginalProjection;
pub use comparison::{compare_original_projections, ProjectionComparisonDenial};
pub mod storage;

pub mod materialization;
pub use materialization::{
    materialize_original_projection, WorthQueryMaterializedFieldValue,
    WorthQueryMaterializedQueryField, WorthQueryMaterializedQueryProjection,
    WorthQueryMaterializedQueryRelation, WorthQueryMaterializedQueryRow,
    WorthQueryProjectionMaterializationRefusal,
};
