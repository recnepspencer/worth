mod plan;
mod tree;
pub(crate) use tree::{RunRange, RunTree};

pub use plan::{ReductionDenial, ReductionPlan};
pub(crate) use tree::ScheduledReductionError;
pub use tree::{
    NativeReductionTree, ReductionAccounting, ReductionEditMemoryBoundError,
    ReductionInsertMemoryBoundError, ReductionMetrics, ReductionRunFailure, ReductionRunStop,
    ReductionShapeCostDenial, ReductionTree,
};

#[cfg(test)]
mod checked_tests;
#[cfg(test)]
mod tests;
