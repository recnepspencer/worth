use worth_store::physical_runtime::{
    PerformedRecoveryPhysicalEffect, RecoveryStagingWriteAction,
};

fn main() {
    let _premature = PerformedRecoveryPhysicalEffect::<RecoveryStagingWriteAction>::record_write(
        todo!(),
    );
}
