//! Selecting route absence retires current custody; it never invents a producer run.
use super::*;
use crate::identity::data::VersionId;
use crate::storage::overlay::record_observation::{
    observe_entity_metadata, observe_relation_metadata,
};
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::primitives::RecordRef;
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalHistoricalOutputAbsence {
    pub(crate) meaning: crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    pub(crate) publication: CommitId,
    pub(crate) generation: VersionId,
    #[serde(skip)]
    _funding: Option<ExecutionMemoryReservation>,
}
impl PartialEq for CanonicalHistoricalOutputAbsence {
    fn eq(&self, other: &Self) -> bool {
        (&self.meaning, self.publication, self.generation)
            == (&other.meaning, other.publication, other.generation)
    }
}
impl Eq for CanonicalHistoricalOutputAbsence {}
impl CanonicalCommitEnvelopeBody {
    pub(crate) fn historical_absent_routes(
        &self,
    ) -> impl Iterator<Item = &CanonicalHistoricalOutputAbsence> {
        self.historical_output_selection
            .iter()
            .flat_map(|event| event.absences.iter())
    }
}
pub(super) fn install(
    original: &CanonicalRequiredOutputOrigin,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    commit: CommitId,
    version: VersionId,
    positions: &mut RequiredOutputPublicationPositions,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CanonicalHistoricalOutputAbsence, TransactionCommitError> {
    let overlay = OverlayStateView::new(basis.inner.root.as_ref(), working);
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
