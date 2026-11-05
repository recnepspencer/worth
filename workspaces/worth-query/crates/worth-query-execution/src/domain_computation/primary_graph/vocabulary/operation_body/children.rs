//! Moved whole from `domain_computation::primary_graph::operation_body::children`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::model::WorthQueryApplicationEffectEntity;
use crate::domain_computation::request_execution::QueryMemoryReservation;

pub(in crate::domain_computation::primary_graph) struct PreparedBodyChildren<S> {
    pub(in crate::domain_computation::primary_graph) targets:
        Vec<WorthQueryApplicationEffectEntity<S, ()>>,
    pub(in crate::domain_computation::primary_graph) _memory: QueryMemoryReservation,
}
