//! Moved whole from `domain_computation::primary_graph::invariant_projection::output_availability_candidate`: the items below and every impl they own.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ObservedOutputPosture {
    Current,
    Pending,
}
