//! The runtime's consumer methods over `domain_computation::primary_graph::record_reservations`; each only calls its subsystem.
use crate::domain_computation::primary_graph::*;
use worth_query_installation::facade::ApplicationSchema;
impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn observe_record_reservations(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        maximum_records: usize,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryRecordReservationObservation, WorthQueryRecordReservationObservationDenial>
    {
        <Self as crate::domain_computation::primary_graph::record_reservations::RecordReservationsOps<Schema>>::observe_record_reservations(self, phase, maximum_records, policy)
    }
}
