//! Actual request and storage causes remain distinct from pure provider refusals.
use worth_execution::{ExecutionRequestArrayDenial, WorkCeilingDenial};

/// Why controlled index work stopped before producing a usable answer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedIndexStop {
    Request(WorkCeilingDenial),
    Storage(ExecutionRequestArrayDenial),
    ValueDecode(
        crate::domain_computation::primary_graph::value_decode::WorthQueryValueDecodeDenial,
    ),
    ForeignNode,
    UnwrittenNode,
    NodeCapacityExceeded,
    InvalidRoot,
    ForeignSourceSlot,
    SourceSlotUnavailable,
    CapacityOverflow,
}

/// A provider's original typed refusal, or an original controlled-work stop.
#[derive(Debug)]
pub enum WorthQueryManagedIndexProviderStop<Domain> {
    Controlled(WorthQueryManagedIndexStop),
    Domain(Domain),
}

impl<D> From<WorthQueryManagedIndexStop> for WorthQueryManagedIndexProviderStop<D> {
    fn from(cause: WorthQueryManagedIndexStop) -> Self {
        Self::Controlled(cause)
    }
}
