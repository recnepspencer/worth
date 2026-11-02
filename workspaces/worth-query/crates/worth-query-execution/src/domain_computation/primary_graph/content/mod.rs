//! Field-bound completed content values and their exact door refusals.
mod admitted;
mod denial;
pub(in crate::domain_computation::primary_graph) use admitted::read_selected;
pub use admitted::AdmittedContent;
pub use denial::{ContentReadDenial, ContentWriteDenial};
pub use worth_relational::facade::content::RelationalContentDenial;

mod staging;
pub(in crate::domain_computation::primary_graph) use staging::{require_contract, ContentStaging};
