use worth_store_recovery_runtime::PerformedRecoveryCleanupRemoval;

fn main() {
    let _forged = PerformedRecoveryCleanupRemoval { performed: fake() };
}

fn fake<T>() -> T {
    panic!("compile-only forged value")
}
