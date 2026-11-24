//! Authenticate and record producer-route absence before publication.
use super::super::super::*;
use super::super::records;
use super::CanonicalHistoricalOutputAbsence;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::RequiredOutputPublicationPositions;
use crate::identity::data::VersionId;
use crate::storage::overlay::record_observation::{
    observe_entity_metadata, observe_relation_metadata,
};
use crate::storage::overlay::{OverlayStateView, WorkingState};
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;

pub(in crate::history::data::canonical_commit_envelope::required_output_origin::historical_selection) fn install(
    original: &CanonicalRequiredOutputOrigin,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    commit: CommitId,
    version: VersionId,
    positions: &mut RequiredOutputPublicationPositions,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CanonicalHistoricalOutputAbsence, TransactionCommitError> {
    let overlay = OverlayStateView::new((*basis.owner_inner()).root.as_ref(), working);
    for member in original
        .members
        .iter()
        .filter(|m| m.posture != crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if observe_entity_metadata(&overlay, member.id, version).is_some() {
            return Err(records::refuse(original.commit.commit_id, member.id));
        }
    }
    for edge in &original.relations {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if observe_relation_metadata(&overlay, edge.id, version).is_some() {
            return Err(TransactionCommitError::publication(crate::publication::data::PublicationError::authored_retirement(
                crate::publication::data::AuthoredRetirementRefusal::HistoricalSelectionNotOwned { publication: original.commit.commit_id }, RecordRef::Relation(edge.id))));
        }
    }
    let bytes = (std::mem::size_of::<CanonicalHistoricalOutputAbsence>()
        + original.meaning.family.len()
        + 32) as u64;
    crate::execution::request_stage::checkpoint(context, bytes).map_err(origin_request)?;
    let funding = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(|stop| origin_request(crate::execution::denial::lease_denial(stop)))?;
    // The route selector's negative coordinate stops cold ancestry at this step.
    // The independent position writer separately clears durable route presence.
    positions.insert(
        &original.meaning,
        crate::history::data::RequiredOutputRoutePosition::HistoricalAbsence(commit),
        lease,
        context,
    )?;
    Ok(CanonicalHistoricalOutputAbsence {
        meaning: original.meaning.clone(),
        publication: original.commit.commit_id,
        generation: original.commit.version_id,
        _funding: Some(funding),
    })
}
