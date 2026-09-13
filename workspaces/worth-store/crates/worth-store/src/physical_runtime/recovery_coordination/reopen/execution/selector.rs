use worth_store_physical_backend::CompletedScheduledRecoveryReopenRead;
use worth_store_physical_format::DurableRootSelector;

use crate::physical_runtime::recovery_coordination::PhysicalRecoveryCoordination;

use super::super::{
    PhysicalRecoveryFreshReopenCommand, PhysicalRecoveryFreshReopenDenial,
    PhysicalRecoveryFreshReopenDenialKind, PhysicalRecoveryFreshReopenStage,
};

pub(super) struct AdmittedSelectorRead {
    pub(super) physical: CompletedScheduledRecoveryReopenRead,
    pub(super) work: crate::physical_runtime::PhysicalWorkIdentity,
    pub(super) signal: crate::physical_runtime::PhysicalSignalSettlementOutcome,
    pub(super) observed: DurableRootSelector,
}

pub(super) fn read_and_admit(
    coordination: &PhysicalRecoveryCoordination,
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryFreshReopenCommand,
    generation: u64,
) -> Result<AdmittedSelectorRead, PhysicalRecoveryFreshReopenDenial> {
    let read = super::read::execute(
        coordination,
        media,
        PhysicalRecoveryFreshReopenStage::CurrentSelector,
        generation,
        worth_store_physical_format::ROOT_SELECTOR_BYTES as u64,
    )
    .map_err(|denial| attach_absence(media, command, denial))?;
    let admitted = super::super::super::source_admission::admit_scheduled_current_selector(
        &read.physical,
        media.store_identity(),
        command.format,
    )
    .map_err(|integrity| invalid_selector(read.physical.clone(), integrity))?;
    let observed = admitted
        .project()
        .map_err(|integrity| invalid_selector(read.physical.clone(), integrity))?;
    coordination
        .root_protocol_counters
        .observe_selector(crate::physical_runtime::PhysicalRootProtocolRoute::ScheduledReopen);
    Ok(AdmittedSelectorRead {
        physical: read.physical,
        work: read.work,
        signal: read.signal,
        observed,
    })
}

fn attach_absence(
    media: &worth_store_physical_backend::AdmittedRecoveryFilesystemMedia,
    command: &PhysicalRecoveryFreshReopenCommand,
    denial: PhysicalRecoveryFreshReopenDenial,
) -> PhysicalRecoveryFreshReopenDenial {
    if matches!(
        denial.kind(),
        PhysicalRecoveryFreshReopenDenialKind::Media(failure)
            if failure.kind() == worth_store_physical_backend::ArtifactTreeFailureKind::Absent
    ) {
        denial.with_integrity(
            crate::physical_runtime::RootProtocolAdmissionDenial::fixed_selector_absent(
                media.store_identity(),
                command.format,
                worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
            ),
        )
    } else {
        denial
    }
}

fn invalid_selector(
    selector: CompletedScheduledRecoveryReopenRead,
    integrity: crate::physical_runtime::RootProtocolAdmissionDenial,
) -> PhysicalRecoveryFreshReopenDenial {
    super::integrity_denial(
        selector,
        None,
        PhysicalRecoveryFreshReopenDenialKind::InvalidSelector,
        integrity,
    )
}
