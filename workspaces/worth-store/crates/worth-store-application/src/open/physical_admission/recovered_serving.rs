//! Admit recovered serving custody after the recovery frame has returned.
use super::{admit_durability, profile, unfinished_outcome, PhysicalOpening, Refusal, Stage};
use std::path::Path;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalRecordOpen, PhysicalRecoveryYieldpointStage,
    PhysicalRuntimeAdmission, PhysicalStore, RecoveredMediaOwnedPhysicalRuntime,
    RecoveredPhysicalServingHandoff, ServingPhysicalRuntime,
};

/// No serving-handoff result storage is retained by the physical recovery caller.
#[inline(never)]
pub(super) fn handoff_from_recovery_outcome(
    outcome: worth_store_recovery_runtime::PhysicalRecoveryOutcome,
) -> Result<RecoveredPhysicalServingHandoff, Refusal> {
    match outcome {
        worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) => handoff
            .into_core()
            .into_serving_handoff()
            .ok_or_else(|| Refusal::new(Stage::Recovery, "selected custody missing")),
        other => Err(super::recovery_refusal::classify(other)),
    }
}

#[inline(never)]
pub(super) fn open(
    root: &Path,
    opening: PhysicalOpening,
    handoff: RecoveredPhysicalServingHandoff,
) -> Result<ServingPhysicalRuntime, Refusal> {
    if let Some(yieldpoint) = &opening.recovery_yieldpoint
        && yieldpoint
            .pause_after(PhysicalRecoveryYieldpointStage::ServingHandoff)
            .is_interrupted()
    {
        return Err(Refusal::new(Stage::Recovery, "serving handoff interrupted"));
    }
    let media = admit_media(root, opening.media, handoff)?;
    open_record_store(media)
}

#[inline(never)]
fn admit_media(
    root: &Path,
    media: FilesystemMediaAdmission,
    handoff: RecoveredPhysicalServingHandoff,
) -> Result<RecoveredMediaOwnedPhysicalRuntime, Refusal> {
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let runtime = PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    match handoff.admit_serving(runtime, media).into_raw() {
        TransitionOutcome::Success(media) => Ok(media),
        other => Err(Refusal::new(
            Stage::MediaAdmission,
            unfinished_outcome(&other),
        )),
    }
}

#[inline(never)]
fn open_record_store(
    media: RecoveredMediaOwnedPhysicalRuntime,
) -> Result<ServingPhysicalRuntime, Refusal> {
    let durability = admit_durability(media.physical_durability_admission_basis())?;
    let open = PhysicalRecordOpen::new(profile::record_format(), profile::access(), durability)
        .with_journal_registry(crate::registry::registry());
    match media.open_record_store(open).into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        TransitionOutcome::Denied(denial) => {
            let reason = format!("{:?}", denial.reason());
            denial.into_runtime().close();
            Err(Refusal::new(Stage::ServingOpen, reason))
        }
        _ => Err(Refusal::new(
            Stage::ServingOpen,
            "serving open after recovery did not complete",
        )),
    }
}
