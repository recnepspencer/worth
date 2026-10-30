pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{lineage_workflow_workspace, LineageEvidenceScenario};
pub use super::operation_lineage::execute;

pub fn reference_intent(role: &str) -> domain::WorthQueryDurableReferenceIntent {
    domain::WorthQueryDurableReferenceIntent::new(
        domain::WorthGraphDurableReferenceKind::PersistentSelection,
        domain::WorthQueryOperationProjectionRole::new(role).unwrap(),
        0,
        0,
    )
}
