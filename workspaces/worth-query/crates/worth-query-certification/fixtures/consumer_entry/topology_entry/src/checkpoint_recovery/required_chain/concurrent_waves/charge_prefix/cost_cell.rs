//! A measured real advancement; the parent always judges both cells.
use super::super::{computation::Failure, history};
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::WorthQueryExecutionPlacementForTest as Placement;

#[cfg(test)]
mod serial {
    use super::*;
    #[test]
    fn failed_advancement_cost_cell() {
        let _guard = super::super::super::super::checkpoint_recovery_test_guard();
        let suffix =
            std::env::var("WORTH_FAILED_PREFIX_SUFFIX").map_or(0, |suffix| suffix.parse().unwrap());
        let observed = history::run_costs(
            Placement::Leased(NonZeroUsize::MIN),
            Some((0, Failure::Complete)),
            [3, suffix],
        );
        println!("\nFAILED_PREFIX_COST:{},{}", observed.work, observed.span);
    }
}
