//! Observe actual cold current ownership independently of the owning scope.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::CommitId;
use crate::identity::data::EntityId;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

pub(crate) fn selected_publication(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
) -> Result<CommitId, TransactionCommitError> {
    let current = super::select_current(execution, basis, &[RecordRef::Entity(owner)])?;
    current
        .origins()
        .find(|origin| origin.meaning.source_scope == owner)
        .map(|origin| origin.commit.commit_id)
        .ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "actual cold ownership court: \"real Query publication absent\"",
            )
        })
}

pub(crate) fn observe_unrelated(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
    unrelated: EntityId,
) -> Result<(), TransactionCommitError> {
    assert!(
        matches!(
            (*basis.owner_inner())
                .root
                .probe_head_region(owner.partition_id),
            crate::storage::residency::PartitionHeadProbe::Cold
        ),
        "the pairing court must reach a genuinely cold current member after recovery"
    );
    // Reading no owning scope or member still validates independently readmitted rows.
    super::select_current(execution, basis, &[RecordRef::Entity(unrelated)]).map(|_| ())
}
