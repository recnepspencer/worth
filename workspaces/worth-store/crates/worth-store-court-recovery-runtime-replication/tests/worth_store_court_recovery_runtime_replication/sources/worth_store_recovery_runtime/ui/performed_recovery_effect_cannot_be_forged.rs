use worth_store::physical_runtime::{PerformedRecoveryPhysicalEffect, RecoveryStagingWriteAction};

fn main() {
    let _forged = PerformedRecoveryPhysicalEffect::<RecoveryStagingWriteAction> {
        evidence: supplied(),
        _action: std::marker::PhantomData,
    };
}

// Type-check the forbidden action without a divergent argument lint.
fn supplied<T>() -> T {
    loop {}
}
