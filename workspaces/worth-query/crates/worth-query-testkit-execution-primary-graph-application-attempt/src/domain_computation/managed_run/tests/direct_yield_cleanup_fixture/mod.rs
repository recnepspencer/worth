pub fn complete_direct_yield_cleanup(
    yielded: worth_query_execution::facade::domain_computation::WorthQueryYieldedDirectRun,
) -> worth_query_execution::facade::domain_computation::WorthQueryDirectYieldCleanupInspection {
    match yielded.cleanup() {
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldCleanupOutcome::Complete(receipt) => {
            receipt.inspection().clone()
        }
        worth_query_execution::facade::domain_computation::WorthQueryDirectYieldCleanupOutcome::RecoveryRequired(_) => {
            panic!("direct provider checkpoint unexpectedly required cleanup recovery")
        }
    }
}
