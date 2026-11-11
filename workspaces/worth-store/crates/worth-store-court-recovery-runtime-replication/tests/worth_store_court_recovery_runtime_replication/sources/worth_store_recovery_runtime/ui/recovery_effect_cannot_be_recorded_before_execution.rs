use worth_store::physical_runtime::{PerformedRecoveryPhysicalEffect, RecoveryStagingWriteAction};

fn main() {
    let _premature =
        PerformedRecoveryPhysicalEffect::<RecoveryStagingWriteAction>::record_write(supplied());
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
