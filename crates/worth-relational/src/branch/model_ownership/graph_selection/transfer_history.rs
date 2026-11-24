//! Recorded navigation participants retain the next inverse's subject, not its owner.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::CanonicalOwnedGraphSelectionOwnerPort as _;
use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

pub(super) fn participants(
    history: CanonicalHistoryRead<'_>,
    execution: &ExecutionRequest<'_, '_>,
    current: &AdmittedRelationalBranchBasis,
    model: EntityId,
    read: &mut basis::BasisRead<'_, '_, '_, '_, '_, '_>,
) -> Result<BTreeSet<EntityId>, Denial> {
    let bound = history.branch.catalog_len();
    let mut next = current.owner_inner().root.commit_id();
    for _ in 0..bound {
        let Some(commit) = next else {
            return Ok(BTreeSet::new());
        };
        read.charge(bound.saturating_add(1))?;
        let envelope = history
            .demand_commit_envelope(execution, commit)
            .map_err(|cause| {
                Denial::Ownership(
                    crate::branch::model_ownership::RelationalModelOwnershipDenial::Native(cause),
                )
            })?
            .ok_or(Denial::TransferEvidenceUnavailable { model, commit })?;
        if let crate::history::data::CanonicalCommitAuthorityKind::OwnedGraphSelection(graph) =
            &envelope.authority_kind
            && graph.model() == model
        {
            let mut participants = BTreeSet::new();
            for entity in graph.transfer_participants() {
                read.retain()?;
                participants.insert(entity);
            }
            return Ok(participants);
        }
        if envelope.commit.parents.len() > 1 {
            return Err(Denial::TransferEvidenceUnavailable { model, commit });
        }
        next = envelope.commit.parents.first().copied();
    }
    match next {
        None => Ok(BTreeSet::new()),
        Some(commit) => Err(Denial::TransferEvidenceUnavailable { model, commit }),
    }
}
