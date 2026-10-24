//! The physical Store under a Relational home: created once, and after that
//! only ever recovered. A home never opens a Store that recovery has not
//! admitted, and never initializes over existing data.

mod profile;

pub(crate) use profile::placement;

use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, MediaOwnedPhysicalRuntime,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore,
    RecoveredPhysicalCheckpointCustody, ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlatformAuthority, WorthStoreRecovery,
};

/// Which physical step refused, so a caller can tell an unusable location
/// from a Store that recovery would not admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RelationalStorePhysicalStage {
    RuntimeAdmission,
    MediaAdmission,
    DurabilityAdmission,
    Initialization,
    RecoveryAdmission,
    Recovery,
    ServingOpen,
}

/// A physical Store step that refused, with the Store's own description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalStorePhysicalRefusal {
    stage: RelationalStorePhysicalStage,
    detail: String,
}

impl RelationalStorePhysicalRefusal {
    fn new(stage: RelationalStorePhysicalStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
        }
    }

    pub fn stage(&self) -> RelationalStorePhysicalStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

type Refusal = RelationalStorePhysicalRefusal;
type Stage = RelationalStorePhysicalStage;

/// Create a new, empty Store at `root`.
pub(crate) fn initialize(root: &Path) -> Result<ServingPhysicalRuntime, Refusal> {
    let media = admit_media(root)?;
    let durability = admit_durability(&media)?;
    let request = PhysicalRecordInitialization::new(
        profile::record_format(),
        profile::placement(),
        profile::access(),
        durability,
    );
    match media.initialize_record_store(request).into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        TransitionOutcome::Denied(denial) => {
            let reason = format!("{:?}", denial.reason());
            denial.into_runtime().close();
            Err(Refusal::new(Stage::Initialization, reason))
        }
        _ => Err(Refusal::new(
            Stage::Initialization,
            "record store initialization did not complete",
        )),
    }
}

/// Recover the Store at `root` through the production recovery entry and
/// open it for serving under the custody recovery established.
pub(crate) fn recover(root: &Path) -> Result<ServingPhysicalRuntime, Refusal> {
    let custody = recover_custody(root)?;
    let media = admit_media(root)?;
    let durability = admit_durability(&media)?;
    let open = PhysicalRecordOpen::new(profile::record_format(), profile::access(), durability);
    let open = match custody {
        Some(custody) => open.with_recovered_checkpoint_custody(custody),
        None => open,
    };
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

fn recover_custody(root: &Path) -> Result<Option<RecoveredPhysicalCheckpointCustody>, Refusal> {
    let limits = PhysicalRecoveryLimits::admit(profile::recovery_limits())
        .map_err(|denial| Refusal::new(Stage::RecoveryAdmission, format!("{denial:?}")))?;
    let configuration = profile::recovery_configuration();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .map_err(|error| Refusal::new(Stage::RecoveryAdmission, format!("{error:?}")))?;
    let backend = authority.qualified_backend_profile().clone();
    let request = PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    );
    match WorthStoreRecovery::recover(request) {
        PhysicalRecoveryOutcome::Recovered(handoff) => {
            Ok(handoff.into_core().into_checkpoint_custody())
        }
        other => Err(Refusal::new(Stage::Recovery, format!("{other:?}"))),
    }
}

fn admit_media(root: &Path) -> Result<MediaOwnedPhysicalRuntime, Refusal> {
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let runtime = PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => Ok(media),
        TransitionOutcome::Denied(denial) => {
            let reason = format!("{:?}", denial.reason());
            denial.into_runtime().close();
            Err(Refusal::new(Stage::MediaAdmission, reason))
        }
        _ => Err(Refusal::new(
            Stage::MediaAdmission,
            "filesystem media admission did not complete",
        )),
    }
}

/// The name of a transition outcome that did not succeed, for refusal
/// details. The typed stage carries the meaning; this only renders it.
pub(crate) fn unfinished_outcome<S, D, Def, St, Reb, F>(
    outcome: &TransitionOutcome<S, D, Def, St, Reb, F>,
) -> &'static str {
    match outcome {
        TransitionOutcome::Success(_) => "succeeded",
        TransitionOutcome::Denied(_) => "denied",
        TransitionOutcome::Deferred(_) => "deferred",
        TransitionOutcome::Stale(_) => "stale",
        TransitionOutcome::RebindRequired(_) => "rebind required",
        TransitionOutcome::Failed(_) => "failed",
    }
}

fn admit_durability(
    media: &MediaOwnedPhysicalRuntime,
) -> Result<worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy, Refusal> {
    let basis = media
        .physical_durability_admission_basis()
        .map_err(|error| Refusal::new(Stage::DurabilityAdmission, format!("{error:?}")))?;
    match profile::durability().admit(basis).into_raw() {
        TransitionOutcome::Success(policy) => Ok(policy),
        other => Err(Refusal::new(
            Stage::DurabilityAdmission,
            unfinished_outcome(&other),
        )),
    }
}
