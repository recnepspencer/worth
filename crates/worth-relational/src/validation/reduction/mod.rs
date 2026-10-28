mod identity;
mod prefix;

pub(crate) use prefix::ReducedInvariantPrefix;
pub(crate) type ReducedInvariants = (
    crate::validation::engine::InvariantExecutionResult,
    crate::authority::commit::preparation::diagnostics::counters::ValidationPreparationCounters,
    Vec<crate::validation::execution::ValidationReducerConflict>,
);
