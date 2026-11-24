pub mod denial;
pub use denial::{
    ProjectionContractRefusal, ProjectionRecordDomain, RelationalProjectionContractDenial,
};
pub mod projection_aspect_filter;
pub mod projection_aspect_scope;
pub mod projection_contract_admission;
pub mod projection_mask_basis;

#[cfg(test)]
mod tests;

pub use projection_aspect_filter::{ProjectionAspectFilter, ProjectionAspectFilterMode};
pub use projection_aspect_scope::{ProjectionAspectRequirement, ProjectionAspectScope};
pub use projection_contract_admission::admit_declared_projection_aspects;
