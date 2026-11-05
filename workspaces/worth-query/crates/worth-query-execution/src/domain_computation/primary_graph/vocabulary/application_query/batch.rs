//! Moved whole from `domain_computation::primary_graph::application_query::batch`: the items below and every impl they own.

/// A scheduled read cannot be admitted to its exact installed batch plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBatchResourceDenial {
    ItemLimit { required: usize, maximum: usize },
    CounterOverflow,
    AllocationFailed,
    ForeignPlan,
    UnplannedRead,
    PlanningFrozen,
}
