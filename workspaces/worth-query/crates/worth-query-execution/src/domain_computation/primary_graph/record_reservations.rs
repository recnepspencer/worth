//! Bounded host observation of actual Native reservations, including refusals.
use super::*;
pub use worth_relational::facade::inspection::{
    RecordReservationObservation as WorthQueryRecordReservationObservation,
    RecordReservationObservationDenial as WorthQueryRecordReservationObservationFailure,
    RecordReservationReport as WorthQueryRecordReservationReport,
};
pub use worth_relational::facade::transactions::RecordRef as WorthQueryReservedRecord;
#[derive(Debug)]
/// Exact phase or native diagnostic-backing refusal; no observer was issued.
pub enum WorthQueryRecordReservationObservationDenial {
    ForeignPhase(WorthQueryForeignAdvancementPhase),
    Observation(WorthQueryRecordReservationObservationFailure),
}
impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static>
    WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Record this runtime's actual reservations on the current thread until
    /// the returned observer is dropped. Its bounded diagnostic backing follows
    /// the explicit allocation policy. Reports include successful reservation
    /// coordinates even if the enclosing proposal is later refused. Other
    /// runtimes, threads and reservations outside the scope are excluded.
    pub fn observe_record_reservations(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        maximum_records: usize,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryRecordReservationObservation, WorthQueryRecordReservationObservationDenial>
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(WorthQueryRecordReservationObservationDenial::ForeignPhase)?;
        self.primary_provider
            .graph
            .with_runtime(&execution, |runtime| {
                runtime
                    .inspect_what_happened()
                    .observe_record_reservations(maximum_records, policy)
                    .map_err(WorthQueryRecordReservationObservationDenial::Observation)
            })
    }
}
