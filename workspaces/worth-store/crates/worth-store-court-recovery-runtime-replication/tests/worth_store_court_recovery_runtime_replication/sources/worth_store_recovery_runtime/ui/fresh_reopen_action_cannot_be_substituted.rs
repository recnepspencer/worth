use worth_store::physical_runtime::{
    PerformedRecoveryPhysicalEffect, RecoveryFreshReopenAction, RecoveryStagingWriteAction,
};

fn substitute(
    effect: PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction>,
) -> PerformedRecoveryPhysicalEffect<RecoveryFreshReopenAction> {
    effect
}

fn main() {}
