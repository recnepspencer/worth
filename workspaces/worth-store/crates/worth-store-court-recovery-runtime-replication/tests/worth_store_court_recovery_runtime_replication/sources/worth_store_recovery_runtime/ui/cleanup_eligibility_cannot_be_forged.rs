use worth_store_recovery_runtime::RecoveryCleanupEligibility;

fn main() {
    let _forged = RecoveryCleanupEligibility {
        covered: fake(),
    };
}

fn fake<T>() -> T {
    panic!("compile-only forged value")
}
