//! Original successor descriptions come from this actual recovery's images.
use contracts::history::data::coordinates::BranchId;
use worth_relational_branch::internal_port::branch::basis_descriptor::{
    RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
};
use worth_relational_branch::internal_port::branch::recorded_basis::RecordedRelationalComponentReference as Recorded;
use worth_relational_branch::internal_port::branch::recorded_basis::RecordedRelationalComponentReferenceOwnerPort as _;
use worth_relational_branch::internal_port::history::commit::index_entry::RelationalCommitIndexEntry;
use worth_relational_contracts::internal_port as contracts;

#[derive(Debug)]
pub enum RelationalRecordedSuccessorRefusal {
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
    ForeignRecoveryRuntime,
    OwnerUnavailable,
    SubjectMismatch,
    MissingRecoveredImage,
    AmbiguousRecoveredImage,
    CanonicalEncoding {
        detail: String,
    },
    Description(
        Box<contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal>,
    ),
}

pub fn select_original_successor(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    branch: &BranchId,
    entry: &RelationalCommitIndexEntry,
    observations: &[RelationalBranchBasisDescriptor],
) -> Result<Recorded, RelationalRecordedSuccessorRefusal> {
    use RelationalRecordedSuccessorRefusal as Refusal;
    let mut selected: Option<Recorded> = None;
    for descriptor in observations {
        if descriptor.branch_id() != branch
            || descriptor.posture() != RelationalBranchBasisPosture::Live
        {
            continue;
        }
        let worth_foundational::FoundationalBranchTarget::Basis(target) =
            descriptor.reference().target()
        else {
            continue;
        };
        if target.selected_commit_id() != entry.commit.commit_id.0
            || target.version_id() != entry.commit.version_id.0
            || target.roots() != &entry.root
            || target.parent_commit_ids().iter().copied().ne(entry
                .commit
                .parents
                .iter()
                .map(|id| id.0))
        {
            continue;
        }
        let candidate = Recorded::from_readmitted_source(execution, descriptor, entry)
            .map_err(|cause| Refusal::Description(Box::new(cause)))?;
        if selected
            .as_ref()
            .is_some_and(|previous| previous != &candidate)
        {
            return Err(Refusal::AmbiguousRecoveredImage);
        }
        selected = Some(candidate);
    }
    selected.ok_or(Refusal::MissingRecoveredImage)
}
