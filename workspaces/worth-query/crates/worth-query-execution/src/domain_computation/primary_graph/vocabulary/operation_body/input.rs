//! Moved whole from `domain_computation::primary_graph::operation_body::input`: the items below and every impl they own.
use crate::domain_computation::request_execution::QueryMemoryReservation;
use worth_foundational::facade::AspectValue;

pub(in crate::domain_computation::primary_graph) struct PreparedBodyInputs {
    pub(in crate::domain_computation::primary_graph) values: Vec<Option<AspectValue>>,
    _reservation: QueryMemoryReservation,
}
