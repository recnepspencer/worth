use worth_store::physical_runtime::{
    PerformedRecoveryPhysicalEffect, RecoveryStagingSynchronizationAction,
    RecoveryStagingWriteAction,
};

fn requires_write(_: PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction>) {}

fn reject_synchronization_as_write(
    synchronization: PerformedRecoveryPhysicalEffect<RecoveryStagingSynchronizationAction>,
) {
    requires_write(synchronization);
}

fn main() {}
