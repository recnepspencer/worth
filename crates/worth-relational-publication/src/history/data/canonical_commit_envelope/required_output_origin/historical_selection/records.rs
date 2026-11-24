//! Selection effects must equal the actual retained producer-owned versions.
use super::super::*;
use contracts::identity::data::primitives::VersionId;
use contracts::identity::data::record_ids::EntityId;
use contracts::identity::data::record_ids::RelationId;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationError;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::MapKernelContext;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::record_observation::{
    observe_entity_metadata, observe_relation_metadata,
};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub(super) fn validate(
    original: &CanonicalRequiredOutputOrigin,
    selected: &AdmittedRelationalBranchBasis,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    version: VersionId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let before = selected.owner_inner().root.as_ref();
    let after = OverlayStateView::new(basis.owner_inner().root.as_ref(), working);
    let original_version = original.commit.version_id;
    let source = original.meaning.source_scope;
    if observe_entity_metadata(&after, source, version).is_none() {
        return Err(refuse(original.commit.commit_id, source));
    }
    for member in original
        .members
        .iter()
        .filter(|m| m.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        contracts::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        let old = observe_entity_metadata(before, member.id, original_version)
            .ok_or_else(|| refuse(original.commit.commit_id, member.id))?;
        let now = observe_entity_metadata(&after, member.id, version)
            .ok_or_else(|| refuse(original.commit.commit_id, member.id))?;
        let payload = old.authoritative_aspect_state();
        contracts::execution::request_stage::checkpoint(
            context,
            payload.map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
        )
        .map_err(origin_request)?;
        if old.kind_id != member.kind
            || now.kind_id != member.kind
            || payload != now.authoritative_aspect_state()
            || now.effective_at != version
        {
            return Err(refuse(original.commit.commit_id, member.id));
        }
    }
    for edge in &original.relations {
        contracts::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        let refusal = || {
            TransactionCommitError::publication(PublicationError::authored_retirement(
                Cause::HistoricalSelectionNotOwned {
                    publication: original.commit.commit_id,
                },
                RecordRef::Relation(edge.id),
            ))
        };
        let old =
            observe_relation_metadata(before, edge.id, original_version).ok_or_else(refusal)?;
        let now = observe_relation_metadata(&after, edge.id, version).ok_or_else(refusal)?;
        let payload = old.authoritative_aspect_state();
        contracts::execution::request_stage::checkpoint(
            context,
            payload.map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
        )
        .map_err(origin_request)?;
        if (old.kind_id, old.endpoints.source, old.endpoints.target)
            != (edge.kind, edge.source, edge.target)
            || (now.kind_id, now.endpoints.source, now.endpoints.target)
                != (edge.kind, edge.source, edge.target)
            || payload != now.authoritative_aspect_state()
            || now.effective_at != version
        {
            return Err(refusal());
        }
    }
    Ok(())
}

pub(super) fn refuse(publication: CommitId, entity: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(PublicationError::authored_retirement(
        Cause::HistoricalSelectionNotOwned { publication },
        RecordRef::Entity(entity),
    ))
}
pub(super) fn entity_version(
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
) -> Option<VersionId> {
    basis
        .owner_inner()
        .root
        .get_current_partition(id.partition_id)?
        .entity_arena()
        .current_metadata_at(id.slot_index())
        .map(|row| row.effective_at)
}
pub(super) fn relation_version(
    basis: &AdmittedRelationalBranchBasis,
    id: RelationId,
) -> Option<VersionId> {
    basis
        .owner_inner()
        .root
        .get_current_partition(id.partition_id)?
        .relation_arena()
        .current_metadata_at(id.slot_index())
        .map(|row| row.effective_at)
}
