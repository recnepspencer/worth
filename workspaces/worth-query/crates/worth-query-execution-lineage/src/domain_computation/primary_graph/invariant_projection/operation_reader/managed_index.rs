//! Native managed-index decision refusal classification.

#[derive(Debug)]
pub enum IndexDecisionStop {
    ContextUnavailable,
    ForeignProjection,
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Native(worth_relational::facade::mvcc::CompanionPreflightStop),
}

impl std::fmt::Display for IndexDecisionStop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for IndexDecisionStop {}

pub fn native_index_decision_cause<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a worth_relational::facade::mvcc::CompanionPreflightStop> {
    match error.downcast_ref::<IndexDecisionStop>()? {
        IndexDecisionStop::Native(cause) => Some(cause),
        _ => None,
    }
}

pub mod failure;
pub use failure::{
    IndexDecisionFailure, WorthQueryComputationManagedIndexDenial,
    WorthQueryComputationManagedIndexDenialKind,
};
