use crate::physical_runtime::{PhysicalPublicationEffect, PhysicalWorkScope};

use super::{
    CompletedPhysicalRecoveryPublicationCommand, PhysicalRecoveryPublicationCommand,
    PhysicalRecoveryPublicationCommandOutcome, PhysicalRecoveryPublicationCommandStage,
};
use crate::physical_runtime::recovery_coordination::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination, RecoveryPublicationOccurrence,
    RecoveryRootProtocolReplacementAction,
};

mod admission;
mod candidate;
mod effect;
mod outcome;

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryPublicationCommand,
) -> PhysicalRecoveryPublicationCommandOutcome {
    let candidates = match candidate::materialize_all(coordination, media, &command) {
        Ok(candidates) => candidates,
        Err(outcome) => return outcome,
    };
    let (candidates, root_protocol) =
        match replace_root_protocol(coordination, media, &command, candidates) {
            Ok(completed) => completed,
            Err(outcome) => return outcome,
        };
    synchronize_record_namespace(coordination, media, command, candidates, root_protocol)
}

fn replace_root_protocol(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryPublicationCommand,
    candidates: Box<[super::CompletedPhysicalRecoveryPublicationCandidate]>,
) -> Result<
    (
        Box<[super::CompletedPhysicalRecoveryPublicationCandidate]>,
        PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
    ),
    PhysicalRecoveryPublicationCommandOutcome,
> {
    let stage = PhysicalRecoveryPublicationCommandStage::RootProtocolReplacement;
    let artifact = command.protocol.catalog_candidate();
    let work = match admission::admit(coordination, stage, PhysicalWorkScope::artifact(artifact)) {
        Ok(work) => work,
        Err(outcome) => return Err(outcome::attach_candidates(outcome, candidates)),
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(denial) => return Err(outcome::pre_effect(stage, denial, candidates, None)),
    };
    let physical = media.replace_recovery_root_protocol_scheduled(
        command.protocol,
        plan.backend_completion_binding()
            .backend_execution_binding(),
    );
    let completed = effect::complete(
        coordination,
        dispatched,
        plan,
        physical,
        stage,
        artifact,
        PhysicalPublicationEffect::ReplaceCatalog,
        candidates,
        None,
    )?;
    let wait = coordination.pause_at(
        crate::physical_runtime::PhysicalRecoveryYieldpointStage::RootProtocolReplacement,
    );
    if wait.is_interrupted() {
        return Err(
            crate::physical_runtime::recovery_coordination::PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
                crate::physical_runtime::recovery_coordination::PhysicalRecoveryPublicationCommandIndeterminate::Yieldpoint {
                    stage,
                    physical: completed.physical,
                    candidates: completed.candidates,
                    root_protocol: completed.root_protocol,
                    wait,
                },
            ),
        );
    }
    let performed =
        PerformedRecoveryPhysicalEffect::record_root_protocol(RecoveryPublicationOccurrence::new(
            coordination.session_identity(),
            command.plan,
            command.staging_generation,
            command.protocol.publication(),
            completed.physical,
            work_identity,
            completed.posture,
            completed.signal,
        ));
    Ok((completed.candidates, performed))
}

fn synchronize_record_namespace(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryPublicationCommand,
    candidates: Box<[super::CompletedPhysicalRecoveryPublicationCandidate]>,
    root_protocol: PerformedRecoveryPhysicalEffect<RecoveryRootProtocolReplacementAction>,
) -> PhysicalRecoveryPublicationCommandOutcome {
    let stage = PhysicalRecoveryPublicationCommandStage::RecordNamespaceSynchronization;
    let artifact = command.protocol.catalog_candidate();
    let work = match admission::admit(coordination, stage, PhysicalWorkScope::artifact(artifact)) {
        Ok(work) => work,
        Err(outcome) => return outcome::attach_effects(outcome, candidates, root_protocol),
    };
    let work_identity = work.intent().identity();
    let (dispatched, plan) = match work.into_execution_parts(None) {
        Ok(parts) => parts,
        Err(denial) => {
            return outcome::pre_effect(stage, denial, candidates, Some(root_protocol));
        }
    };
    let physical = media.synchronize_recovery_record_namespace_scheduled(
        plan.backend_completion_binding()
            .backend_execution_binding(),
    );
    let completed = match effect::complete(
        coordination,
        dispatched,
        plan,
        physical,
        stage,
        artifact,
        PhysicalPublicationEffect::SynchronizeRecordFamily,
        candidates,
        Some(root_protocol),
    ) {
        Ok(completed) => completed,
        Err(outcome) => return outcome,
    };
    let root_protocol = completed
        .root_protocol
        .expect("namespace completion retains root-protocol authority");
    let wait = coordination.pause_at(
        crate::physical_runtime::PhysicalRecoveryYieldpointStage::RecordNamespaceSynchronization,
    );
    if wait.is_interrupted() {
        return crate::physical_runtime::recovery_coordination::PhysicalRecoveryPublicationCommandOutcome::Indeterminate(
            crate::physical_runtime::recovery_coordination::PhysicalRecoveryPublicationCommandIndeterminate::Yieldpoint {
                stage,
                physical: completed.physical,
                candidates: completed.candidates,
                root_protocol: Some(root_protocol),
                wait,
            },
        );
    }
    let record_namespace = PerformedRecoveryPhysicalEffect::record_record_namespace(
        RecoveryPublicationOccurrence::new(
            coordination.session_identity(),
            command.plan,
            command.staging_generation,
            command.protocol.publication(),
            completed.physical,
            work_identity,
            completed.posture,
            completed.signal,
        ),
    );
    PhysicalRecoveryPublicationCommandOutcome::Completed(
        CompletedPhysicalRecoveryPublicationCommand::new(
            completed.candidates,
            root_protocol,
            record_namespace,
        ),
    )
}
