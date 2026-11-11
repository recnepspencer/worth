//! Qualify the source and return recovery before constructing serving owners.
use super::{PhysicalOpening, Refusal, Stage};
use std::path::Path;
use worth_store::physical_runtime::{QualifiedRecoveryFilesystemMedia, RecoveryStorePresence};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAdmissionError,
};

pub(super) enum PreparedPhysicalHome {
    Fresh {
        held: QualifiedRecoveryFilesystemMedia,
        opening: PhysicalOpening,
    },
    Recovered {
        outcome: PhysicalRecoveryOutcome,
        opening: PhysicalOpening,
    },
}

#[inline(never)]
pub(super) fn prepare(
    root: &Path,
    opening: PhysicalOpening,
) -> Result<PreparedPhysicalHome, Refusal> {
    super::require_no_discard(root)?;
    std::fs::create_dir_all(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, error.to_string()))?;
    let held = opening.media.qualify_home(root).map_err(|error| {
        if let Err(refusal) = super::require_no_discard(root) {
            return refusal;
        }
        super::recovery_admission_refusal(
            PhysicalRecoveryPlatformAdmissionError::BackendQualification(error),
        )
    })?;
    super::require_no_discard(root)?;
    match held
        .store_presence()
        .map_err(|error| Refusal::new(Stage::RecoveryAdmission, format!("{error:?}")))?
    {
        RecoveryStorePresence::ProvenAbsent => Ok(PreparedPhysicalHome::Fresh { held, opening }),
        RecoveryStorePresence::ExistingArtifacts => {
            let request = super::prepare_recovery_request(root, held)?;
            let outcome = super::recover_physical(request, opening.recovery_yieldpoint.clone());
            Ok(PreparedPhysicalHome::Recovered { outcome, opening })
        }
    }
}
