use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;
use worth_store_physical_format::DurablePhysicalRootManifest;

use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

use super::super::{
    PhysicalRecoveryFreshReopenCommand, PhysicalRecoveryFreshReopenDenial,
    PhysicalRecoveryFreshReopenDenialKind, PhysicalRecoveryFreshReopenStage,
};

pub(super) struct AdmittedRootManifestRead {
    pub(super) physical: CompletedScheduledRecoveryReopenRead,
    pub(super) work: crate::physical_runtime::PhysicalWorkIdentity,
    pub(super) signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    pub(super) observed: DurablePhysicalRootManifest,
}

pub(super) fn read_and_admit(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryFreshReopenCommand,
    generation: u64,
    selector: &CompletedScheduledRecoveryReopenRead,
) -> Result<AdmittedRootManifestRead, PhysicalRecoveryFreshReopenDenial> {
    let root_bytes = command.expected_root.encode(command.format).len() as u64;
    let read = super::read::execute(
        coordination,
        media,
        PhysicalRecoveryFreshReopenStage::RootManifest,
        generation,
        root_bytes,
    )
    .map_err(|denial| attach_selector_and_absence(media, command, selector, root_bytes, denial))?;
    let admitted = super::super::super::source_admission::admit_scheduled_root_manifest(
        &read.physical,
        media.store_identity(),
        command.format,
        generation,
    )
    .map_err(|integrity| invalid_root(selector.clone(), read.physical.clone(), integrity))?;
    let observed = admitted
        .project()
        .map_err(|integrity| invalid_root(selector.clone(), read.physical.clone(), integrity))?;
    coordination
        .root_protocol_counters
        .observe_root(crate::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen);
    Ok(AdmittedRootManifestRead {
        physical: read.physical,
        work: read.work,
        signal: read.signal,
        observed,
    })
}

fn attach_selector_and_absence(
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryFreshReopenCommand,
    selector: &CompletedScheduledRecoveryReopenRead,
    root_bytes: u64,
    mut denial: PhysicalRecoveryFreshReopenDenial,
) -> PhysicalRecoveryFreshReopenDenial {
    denial.selector = Some(selector.clone());
    if matches!(
        denial.kind(),
        PhysicalRecoveryFreshReopenDenialKind::Media(failure)
            if failure.kind() == worth_store_physical_backend::ArtifactTreeFailureKind::Absent
    ) {
        denial.with_integrity(
            crate::physical_runtime::RootProtocolAdmissionDenial::addressed_root_absent(
                media.store_identity(),
                command.format,
                command.expected_root.generation(),
                root_bytes,
            ),
        )
    } else {
        denial
    }
}

fn invalid_root(
    selector: CompletedScheduledRecoveryReopenRead,
    root: CompletedScheduledRecoveryReopenRead,
    integrity: crate::physical_runtime::RootProtocolAdmissionDenial,
) -> PhysicalRecoveryFreshReopenDenial {
    super::integrity_denial(
        selector,
        Some(root),
        PhysicalRecoveryFreshReopenDenialKind::InvalidRoot,
        integrity,
    )
}
