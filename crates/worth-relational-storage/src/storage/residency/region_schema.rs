//! The exact schema needed to verify and materialize one retained region.
use crate::schema::data::{AspectContractPlanCatalog, RelationalSchemaRegistry};
use worth_foundational::facade::AspectContract;

/// A branch root may retain additional authority. Region storage borrows only
/// the immutable compilation inputs, through the same original allocation.
pub trait RelationalRegionSchema: std::fmt::Debug + Send + Sync {
    fn registry(&self) -> &RelationalSchemaRegistry;
    fn aspect_plans(&self) -> &AspectContractPlanCatalog;
    fn retained_aspect_contracts(&self) -> &[AspectContract];
}
