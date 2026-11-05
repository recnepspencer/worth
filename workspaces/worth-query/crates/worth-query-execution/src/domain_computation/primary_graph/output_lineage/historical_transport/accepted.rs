//! The exact transported row retained by restored-demand acceptance.
use super::super::*;

#[derive(Clone)]
pub struct HistoricalAcceptedOrigin {
    row: Arc<OnceLock<RecordedOutput>>,
}
impl HistoricalAcceptedOrigin {
    fn row(&self) -> &RecordedOutput {
        self.row
            .get()
            .expect("acceptance retains a filled transported row")
    }
    pub(in crate::domain_computation::primary_graph) fn same_owner(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.row, &other.row)
    }
    pub(in crate::domain_computation::primary_graph) fn publication(
        &self,
    ) -> &worth_relational::facade::mvcc::RelationalRequiredOutputPublication {
        self.row()
            .required_publication
            .as_ref()
            .expect("historical acceptance retains its committed publication")
    }
    pub(in crate::domain_computation::primary_graph) fn settlement(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.row().settlement_identity
    }
    pub(in crate::domain_computation::primary_graph) fn source_identity(
        &self,
    ) -> Option<RecordedSourceIdentity> {
        self.row().source_identity
    }
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> [u8; 32] {
        self.row().idempotency_key_identity
    }
    pub(in crate::domain_computation::primary_graph) fn implementation(
        &self,
    ) -> worth_relational::facade::mvcc::RequiredOutputProducingImplementation {
        self.row()
            .producing_implementation
            .expect("verified historical row has a producing build")
    }
    pub(in crate::domain_computation::primary_graph) fn computation_source(
        &self,
    ) -> ComputationSourceEvidence {
        self.row().computation_source
    }
    pub(in crate::domain_computation::primary_graph) fn dependency(&self) -> Option<[u8; 32]> {
        self.row().producer_dependency_identity
    }
}
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn historical_accepted_origin(
        &self,
        candidate: &current_output::retained_candidates::WorthQueryRetainedOutputCandidate,
    ) -> Option<HistoricalAcceptedOrigin> {
        let pin = &candidate.settlement_identity;
        let coordinate = pin.coordinate();
        let cell = self
            .by_source
            .get(pin.source())?
            .get(&coordinate.occurrence)?
            .get(&coordinate.generation)?
            .get(pin.slot())?;
        let row = cell.get()?;
        (Arc::ptr_eq(&row.settlement_identity, pin)
            && row.idempotency_key_identity == candidate.idempotency_key_identity
            && row.historical_selection.is_some()
            && row.producing_implementation.is_some()
            && row
                .required_publication
                .as_ref()
                .is_some_and(|publication| publication.is_historical_selection()))
        .then(|| HistoricalAcceptedOrigin {
            row: Arc::clone(cell),
        })
    }
}
