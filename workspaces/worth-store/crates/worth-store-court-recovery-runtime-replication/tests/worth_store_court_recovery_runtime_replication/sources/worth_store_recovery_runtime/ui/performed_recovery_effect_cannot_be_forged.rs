use worth_store::physical_runtime::{
    PerformedRecoveryPhysicalEffect, RecoveryStagingWriteAction,
};

fn main() {
    let _forged = PerformedRecoveryPhysicalEffect::<RecoveryStagingWriteAction> {
        evidence: todo!(),
        _action: std::marker::PhantomData,
    };
}
