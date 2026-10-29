use worth_store_recovery_runtime::RecoveryCleanupEligibility;

fn main() {
    let eligibility = fake::<RecoveryCleanupEligibility>();
    let _reused = eligibility.clone();
}

fn fake<T>() -> T {
    panic!("compile-only eligibility value")
}
