//! Moved whole from `domain_computation::primary_graph::application_query::managed_index::predicate::tree`: the items below and every impl they own.

/// Issued by the registration owner; equal probes retain distinct exact slots.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WatchSlot(
    pub(in crate::domain_computation::primary_graph) u64,
);
