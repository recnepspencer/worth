use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

use super::{PhysicalRecoveryStagingCommand, PhysicalRecoveryStagingCommandOutcome};

mod admission;
mod materialization;
mod outcome;
mod synchronization;

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryStagingCommand<'_>,
) -> PhysicalRecoveryStagingCommandOutcome {
    let materialization = match materialization::execute(coordination, media, &command) {
        Ok(materialization) => materialization,
        Err(outcome) => return outcome,
    };
    synchronization::execute(coordination, media, command, materialization)
}
