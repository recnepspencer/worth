use worth_store::physical_runtime::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryStagingCommand,
    RecoveryStagingWriteAction,
};

fn requires_performed(_: PerformedRecoveryPhysicalEffect<RecoveryStagingWriteAction>) {}

fn reject_admitted_staging_attempt(attempt: PhysicalRecoveryStagingCommand) {
    requires_performed(attempt);
}

fn main() {}
