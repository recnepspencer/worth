//! Authenticate and record producer-route absence before publication.
use worth_relational_contracts::internal_port as contracts;
use super::super::super::*;
use super::super::records;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::historical_selection::absence::CanonicalHistoricalOutputAbsence;
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _};
use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputPublicationPositions;
use contracts::identity::data::primitives::VersionId;
use worth_relational_storage::internal_port::storage::overlay::record_observation::{observe_entity_metadata, observe_relation_metadata};
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionMemoryReservation, ExecutionResourceLease, MapKernelContext};
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;

pub(in crate::internal_port::history::data::canonical_commit_envelope::required_output_origin::historical_selection) fn install(
    original: &CanonicalRequiredOutputOrigin,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    commit: CommitId,
    version: VersionId,
    positions: &mut RequiredOutputPublicationPositions,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CanonicalHistoricalOutputAbsence, TransactionCommitError> {
    let overlay = OverlayStateView::new(basis.owner_inner().root.as_ref(), working);
    for member in original
        .members
        .iter()
        .filter(|m| m.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        contracts::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if observe_entity_metadata(&overlay, member.id, version).is_some() {
            return Err(records::refuse(original.commit.commit_id, member.id));
        }
    }
    for edge in &original.relations {
        contracts::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if observe_relation_metadata(&overlay, edge.id, version).is_some() {
            return Err(TransactionCommitError::publication(contracts::publication::data::publication_error::PublicationError::authored_retirement(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::HistoricalSelectionNotOwned { publication: original.commit.commit_id }, RecordRef::Relation(edge.id))));
        }
    }
    let bytes = (std::mem::size_of::<CanonicalHistoricalOutputAbsence>()
        + original.meaning.family.len()
        + 32) as u64;
    contracts::execution::request_stage::checkpoint(context, bytes).map_err(origin_request)?;
    let funding = ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
        .map_err(|stop| origin_request(contracts::execution::denial::lease_denial(stop)))?;
    // The route selector's negative coordinate stops cold ancestry at this step.
    // The independent position writer separately clears durable route presence.
    positions.insert(
        &original.meaning,
        worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition::HistoricalAbsence(commit),
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
