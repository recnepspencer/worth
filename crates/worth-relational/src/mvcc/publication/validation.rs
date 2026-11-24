use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::reference::RelationalBranchReferenceCell;
use crate::branch::target::RelationalBranchTarget;
use crate::history::data::BranchId;
use crate::history::data::{CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use worth_foundational::FoundationalBranchTarget;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

#[derive(Debug, Clone, Copy)]
pub(crate) enum PublicationSequence {
    Truth,
    RecoveryTruth,
}

pub(crate) struct PublicationRequest<'a> {
    pub(crate) commit_id: CommitId,
    pub(crate) commit_reference: &'a RelationalCommitReceipt,
    pub(crate) binding: &'a AdmittedRelationalBranchBasis,
    pub(crate) envelope: &'a CanonicalCommitEnvelope,
    pub(crate) sequence: PublicationSequence,
}

pub(crate) struct ValidatedPublication {
    pub(crate) branch_id: BranchId,
    pub(crate) next_cell: RelationalBranchReferenceCell,
    pub(super) sequence_successors:
        crate::runtime::state::subsystems::history::PreparedCommitSequence,
}

fn validate_publication_identity(request: &PublicationRequest<'_>) -> Result<(), String> {
    let branch_id = request.binding.identity().branch_id();
    if request.commit_reference.commit_id != request.commit_id {
        return Err(format!(
            "publication commit identity mismatch: expected {}, got {}",
            request.commit_id.0, request.commit_reference.commit_id.0
        ));
    }
    if request.envelope.commit != *request.commit_reference {
        return Err("publication envelope commit identity mismatch".to_owned());
    }
    if &request.commit_reference.branch_id != branch_id {
        return Err(format!(
            "publication branch identity mismatch: expected `{}`, got `{}`",
            branch_id.0, request.commit_reference.branch_id.0
        ));
    }
    if &request.envelope.branch_context != branch_id {
        return Err(format!(
            "publication envelope branch context mismatch: expected `{}`, got `{}`",
            branch_id.0, request.envelope.branch_context.0
        ));
    }
    if !request.binding.is_current() {
        return Err("publication owner binding is foreign or stale".to_owned());
    }
    if request
        .commit_reference
        .version_id
        .0
        .checked_add(1)
        .is_none()
    {
        return Err("version id sequence overflow".to_owned());
    }
    if request.commit_id.0.checked_add(1).is_none() {
        return Err("commit id sequence overflow".to_owned());
    }
    Ok(())
}

pub(crate) fn validate_publication(
    history: &HistorySubsystem,
    request: PublicationRequest<'_>,
) -> Result<ValidatedPublication, String> {
    let branch_id = request.binding.identity().branch_id();
    validate_publication_identity(&request)?;
    let catalog_admission = match request.sequence {
        PublicationSequence::Truth => history.validate_new_catalog_envelope(request.envelope),
        PublicationSequence::RecoveryTruth => history.validate_catalog_envelope(request.envelope),
    };
    catalog_admission
        .map_err(|denial| format!("publication catalog admission denied: {denial:?}"))?;
    advance_publication_head(
        branch_id,
        history
            .branch_cell(branch_id)
            .map(|cell| cell.clone_for_head_replacement()),
        history.runtime_instance_id,
        request.commit_reference,
    )
}

pub(crate) fn validate_prepared_publication(
    history: &RelationalPreparationHistory,
    runtime_instance_id: u64,
    request: PublicationRequest<'_>,
) -> Result<ValidatedPublication, String> {
    let branch_id = request.binding.identity().branch_id();
    validate_publication_identity(&request)?;
    match request.sequence {
        PublicationSequence::Truth => history.validate_new_publication_envelope(request.envelope),
        PublicationSequence::RecoveryTruth => {
            history.validate_recovery_publication_envelope(request.envelope)
        }
    }?;
    advance_publication_head(
        branch_id,
        history
            .branch_cell(branch_id)
            .map(|cell| cell.clone_for_head_replacement()),
        runtime_instance_id,
        request.commit_reference,
    )
}

fn advance_publication_head(
    branch_id: &BranchId,
    cell: Option<RelationalBranchReferenceCell>,
    runtime_instance_id: u64,
    commit: &RelationalCommitReceipt,
) -> Result<ValidatedPublication, String> {
    let cell = cell.ok_or_else(|| {
        format!(
            "publication cannot mint a missing branch cell `{}`",
            branch_id.0
        )
    })?;
    let roots = RelationalBranchTarget::roots_for_commit(commit);
    let target = RelationalBranchTarget::from_commit_receipt(runtime_instance_id, commit, roots);
    cell.advance_truth(FoundationalBranchTarget::basis(target))
        .map_err(|denial| format!("publication branch advance denied: {denial:?}"))?;
    Ok(ValidatedPublication {
        branch_id: branch_id.clone(),
        next_cell: cell,
        sequence_successors:
            crate::runtime::state::subsystems::history::PreparedCommitSequence::new(commit)?,
    })
}
