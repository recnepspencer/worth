use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;

use crate::physical_runtime::recovery_coordination::{
    PerformedRecoveryPhysicalEffect, PhysicalRecoveryCoordination, RecoveryFreshReopenOccurrence,
};

use super::{
    CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryFreshReopenCommand,
    PhysicalRecoveryFreshReopenDenial, PhysicalRecoveryFreshReopenDenialKind,
    PhysicalRecoveryFreshReopenOutcome, PhysicalRecoveryFreshReopenStage,
};

mod read;
mod root_manifest;
mod selector;

pub(super) fn execute(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: PhysicalRecoveryFreshReopenCommand,
) -> PhysicalRecoveryFreshReopenOutcome {
    let generation = command.expected_root.generation();
    let selector = match selector::read_and_admit(coordination, media, &command, generation) {
        Ok(selector) => selector,
        Err(denial) => return PhysicalRecoveryFreshReopenOutcome::Denied(denial),
    };
    if selector.observed != command.expected_selector {
        return denied_binding(
            selector.physical,
            None,
            PhysicalRecoveryFreshReopenDenialKind::BindingMismatch,
        );
    }
    let root = match root_manifest::read_and_admit(
        coordination,
        media,
        &command,
        selector.observed.root_generation(),
        &selector.physical,
    ) {
        Ok(root) => root,
        Err(denial) => return PhysicalRecoveryFreshReopenOutcome::Denied(denial),
    };
    if root.observed != command.expected_root {
        return denied_binding(
            selector.physical,
            Some(root.physical),
            PhysicalRecoveryFreshReopenDenialKind::BindingMismatch,
        );
    }
    let wait = coordination.pause_at(
        crate::physical_runtime::PhysicalRecoveryYieldpointStage::FreshReopenExactBinding,
    );
    if wait.is_interrupted() {
        return PhysicalRecoveryFreshReopenOutcome::Denied(PhysicalRecoveryFreshReopenDenial::new(
            PhysicalRecoveryFreshReopenStage::ExactBinding,
            PhysicalRecoveryFreshReopenDenialKind::Yieldpoint(wait),
            Some(selector.physical.clone()),
            Some(root.physical.clone()),
            None,
        ));
    }
    let performed =
        PerformedRecoveryPhysicalEffect::record_fresh_reopen(RecoveryFreshReopenOccurrence::new(
            coordination.session_identity(),
            command.plan,
            generation,
            selector.physical,
            root.physical,
            selector.work,
            root.work,
            selector.signal,
            root.signal,
        ));
    PhysicalRecoveryFreshReopenOutcome::Completed(CompletedPhysicalRecoveryFreshReopen::new(
        root.observed,
        command.format,
        performed,
    ))
}

fn denied_binding(
    selector: CompletedScheduledRecoveryReopenRead,
    root: Option<CompletedScheduledRecoveryReopenRead>,
    kind: PhysicalRecoveryFreshReopenDenialKind,
) -> PhysicalRecoveryFreshReopenOutcome {
    PhysicalRecoveryFreshReopenOutcome::Denied(PhysicalRecoveryFreshReopenDenial::new(
        PhysicalRecoveryFreshReopenStage::ExactBinding,
        kind,
        Some(selector),
        root,
        None,
    ))
}

pub(super) fn integrity_denial(
    selector: CompletedScheduledRecoveryReopenRead,
    root: Option<CompletedScheduledRecoveryReopenRead>,
    kind: PhysicalRecoveryFreshReopenDenialKind,
    integrity: crate::physical_runtime::RootProtocolAdmissionDenial,
) -> PhysicalRecoveryFreshReopenDenial {
    PhysicalRecoveryFreshReopenDenial::new(
        PhysicalRecoveryFreshReopenStage::ExactBinding,
        kind,
        Some(selector),
        root,
        None,
    )
    .with_integrity(integrity)
}
