pub use super::installed_operation_fixture::{
    artifact_lease_workspace, artifact_move_workspace, bind_artifact_workflow, lease_intent,
    lease_intent_with_mode, move_intent,
};
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub fn assert_owner_counts(
    snapshot: domain::WorthQueryArtifactOwnerSnapshot,
    owners: usize,
    borrows: usize,
    leases: usize,
) {
    assert_eq!(snapshot.owner_count(), owners);
    assert_eq!(snapshot.borrow_count(), borrows);
    assert_eq!(snapshot.lease_count(), leases);
}
