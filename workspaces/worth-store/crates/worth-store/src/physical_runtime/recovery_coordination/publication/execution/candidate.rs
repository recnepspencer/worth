use crate::physical_runtime::recovery_coordination::{
    CompletedPhysicalRecoveryPublicationCandidate, PhysicalRecoveryCoordination,
    PhysicalRecoveryPublicationCandidate,
};

use super::super::{PhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommandOutcome};

mod materialization;
mod synchronization;

pub(super) fn materialize_all(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryPublicationCommand,
) -> Result<
    Box<[CompletedPhysicalRecoveryPublicationCandidate]>,
    PhysicalRecoveryPublicationCommandOutcome,
> {
    // Materialize in reverse canonical artifact order (bottom-up: membership
    // and routing blocks -> FreeSpaceManifest -> RootManifest -> protocol
    // candidates) so an OS kill after RootManifest is durable can never leave
    // referenced child blocks absent on the next recovery reopen.
    let mut completed = Vec::with_capacity(command.candidates.len());
    for (ordinal, candidate) in command.candidates.iter().rev().enumerate() {
        let materialization = materialization::execute(
            coordination,
            media,
            command,
            candidate,
            ordinal as u64,
            completed,
        )?;
        let (settled, completed_candidate) = synchronization::execute(
            coordination,
            media,
            command,
            candidate,
            ordinal as u64,
            materialization,
        )?;
        completed = settled;
        completed.push(completed_candidate);
    }
    Ok(completed.into_boxed_slice())
}

pub(super) fn occurrence(
    coordination: &PhysicalRecoveryCoordination,
    command: &PhysicalRecoveryPublicationCommand,
    candidate: &PhysicalRecoveryPublicationCandidate,
    ordinal: u64,
    work: crate::physical_runtime::PhysicalWorkIdentity,
    scheduler: crate::physical_runtime::PhysicalWorkSchedulerPosture,
    signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
) -> crate::physical_runtime::recovery_coordination::RecoveryPublicationCandidateOccurrence {
    crate::physical_runtime::recovery_coordination::RecoveryPublicationCandidateOccurrence::new(
        coordination.session_identity(),
        command.plan,
        command.generation.generation(),
        command.protocol.publication(),
        candidate.artifact(),
        ordinal,
        work,
        scheduler,
        signal,
    )
}

pub(super) fn scheduler_posture(
    outcome: &worth_store_io_scheduler::QueueExecutionOutcome,
) -> crate::physical_runtime::PhysicalWorkSchedulerPosture {
    if matches!(
        outcome,
        worth_store_io_scheduler::QueueExecutionOutcome::Executed(_)
    ) {
        crate::physical_runtime::PhysicalWorkSchedulerPosture::Executed
    } else {
        crate::physical_runtime::PhysicalWorkSchedulerPosture::RejectedAfterEffect
    }
}
