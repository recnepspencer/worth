//! Field-bound completed content values and their exact door refusals.
mod admitted;
mod resolution;
mod selection;
pub(in crate::domain_computation::primary_graph) use resolution::{resolve_frame, typed};
pub(in crate::domain_computation::primary_graph) use selection::{
    ContentFieldSelection, DecodeReference,
};
mod denial;
pub use admitted::AdmittedContent;
pub use denial::{ContentReadDenial, ContentWriteDenial};
pub use worth_relational::facade::content::{RelationalContentDenial, RelationalContentRejection};

mod staging;
pub(in crate::domain_computation::primary_graph) use staging::{require_contract, ContentStaging};

mod retention_work;
pub(in crate::domain_computation) use retention_work::checkpoint as retention_checkpoint;
