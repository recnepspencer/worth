//! Pure batch resource denial enum.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryBatchResourceDenial {
    ItemLimit { required: usize, maximum: usize },
    CounterOverflow,
    AllocationFailed,
    ForeignPlan,
    UnplannedRead,
    PlanningFrozen,
}
