use worth_store_recovery_runtime::StagedPhysicalRecovery;

fn main() {
    let _forged = StagedPhysicalRecovery {
        authority: supplied(),
        coordination: supplied(),
        selection: supplied(),
        discovery_counters: supplied(),
        freshness: supplied(),
        fates: supplied(),
        planning_counters: supplied(),
        base: supplied(),
        publication: supplied(),
        quiescence: supplied(),
        closed: supplied(),
        staging_counters: supplied(),
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
