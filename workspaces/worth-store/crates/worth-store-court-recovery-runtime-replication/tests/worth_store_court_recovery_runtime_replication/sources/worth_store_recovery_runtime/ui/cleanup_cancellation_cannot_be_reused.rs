use worth_store_recovery_runtime::{
    PhysicalRecoveryCleanupCancellation, ReopenedPhysicalRecovery,
};

fn reuse(
    first: ReopenedPhysicalRecovery,
    second: ReopenedPhysicalRecovery,
    cancellation: PhysicalRecoveryCleanupCancellation,
) {
    let _ = first.finish_with_cleanup_cancellation(cancellation);
    let _ = second.finish_with_cleanup_cancellation(cancellation);
}

fn main() {}
