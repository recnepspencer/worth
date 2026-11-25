//! Observe actual cold current ownership independently of the owning scope.
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess as _;

pub fn selected_publication(
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
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "actual cold ownership court: \"real Query publication absent\"",
            )
        })
}

pub fn observe_unrelated(
    execution: &ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    owner: EntityId,
    unrelated: EntityId,
) -> Result<(), TransactionCommitError> {
    assert!(
        matches!(
            basis
                .probe_head_partition(owner.partition_id),
            worth_relational_storage::internal_port::storage::residency::head_probe::PartitionHeadProbe::Cold
        ),
        "the pairing court must reach a genuinely cold current member after recovery"
    );
    // Reading no owning scope or member still validates independently readmitted rows.
    super::select_current(execution, basis, &[RecordRef::Entity(unrelated)]).map(|_| ())
}
