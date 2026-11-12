//! Selecting route absence retires current custody; it never invents a producer run.
use super::*;
use crate::{
    identity::data::VersionId, storage::overlay::OverlayStateView, transactions::data::RecordRef,
    validation::engine::state_view::InvariantStateView,
};
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalHistoricalOutputAbsence {
    pub(crate) meaning: crate::mvcc::RequiredOutputPublicationMeaning,
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
    selection: &RelationalHistoricalOutputSelection,
    basis: &AdmittedRelationalBranchBasis,
    working: &WorkingState,
    commit: CommitId,
    version: VersionId,
    positions: &mut RequiredOutputPublicationPositions,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<CanonicalHistoricalOutputAbsence, TransactionCommitError> {
    let original = &selection.publication.origin;
    let overlay = OverlayStateView::new(basis.inner.root.as_ref(), working);
    let after = InvariantStateView::new(&overlay, version);
    for member in original
        .members
        .iter()
        .filter(|m| m.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
    {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if after.entity_metadata(member.id).is_some() {
            return Err(records::refuse(original.commit.commit_id, member.id));
        }
    }
    for edge in &original.relations {
        crate::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
        if after.relation_metadata(edge.id).is_some() {
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
