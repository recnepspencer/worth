//! Moved whole from `domain_computation::primary_graph::operation_body::source`: the items below and every impl they own.
use crate::domain_computation::request_execution::QueryMemoryReservation;
use worth_foundational::facade::AspectValue;

pub(in crate::domain_computation::primary_graph) struct PreparedBodySource {
    pub(in crate::domain_computation::primary_graph) value: AspectValue,
    pub(in crate::domain_computation::primary_graph) _reservation: QueryMemoryReservation,
}

impl PreparedBodySource {
    pub(in crate::domain_computation::primary_graph) fn new(
        value: AspectValue,
        reservation: QueryMemoryReservation,
    ) -> Self {
        Self {
            value,
            _reservation: reservation,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (AspectValue, QueryMemoryReservation) {
        (self.value, self._reservation)
    }
}
