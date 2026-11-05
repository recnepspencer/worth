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
pub trait RecordReservationsOps<
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
>:
    crate::domain_computation::primary_graph::record_reservations::host::RecordReservationsHost<Schema>
{
    fn observe_record_reservations(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        maximum_records: usize,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryRecordReservationObservation, WorthQueryRecordReservationObservationDenial>
    {
        let execution = phase
            .execution_request_for(self.product_runtime())
            .map_err(WorthQueryRecordReservationObservationDenial::ForeignPhase)?;
        self.provider_graph().with_runtime(&execution, |runtime| {
            runtime
                .inspect_what_happened()
                .observe_record_reservations(maximum_records, policy)
                .map_err(WorthQueryRecordReservationObservationDenial::Observation)
        })
    }
}

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, HostRuntime: crate::domain_computation::primary_graph::record_reservations::host::RecordReservationsHost<Schema> + ?Sized> RecordReservationsOps<Schema> for HostRuntime {}
pub(crate) mod host;
