//! The physical Store under a Relational home: created once, and after that
//! only ever recovered. A home never opens a Store that recovery has not
//! admitted, and never initializes over existing data.

mod profile;
mod recovery_refusal;

pub(crate) use profile::placement;
#[cfg(feature = "certification-test-authority")]
pub use profile::RelationalStoreCertificationProfile;

use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, MediaOwnedPhysicalRuntime,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRecoveryProcessYieldpoint,
    PhysicalRuntimeAdmission, PhysicalStore, RecoveredPhysicalCheckpointCustody,
    ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlatformAdmissionError, PhysicalRecoveryPlatformAuthority, WorthStoreRecovery,
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
    absent: bool,
    kind: RelationalStorePhysicalRefusalKind,
}

impl RelationalStorePhysicalRefusal {
    fn new(stage: RelationalStorePhysicalStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
            absent: false,
            kind: RelationalStorePhysicalRefusalKind::PhysicalStep,
        }
    }

    pub fn kind(&self) -> RelationalStorePhysicalRefusalKind {
        self.kind
    }

    pub fn stage(&self) -> RelationalStorePhysicalStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalStorePhysicalRefusalKind {
    PhysicalStep,
    MutationOwnerContended,
    UnsupportedRootSchema { observed: u32 },
}

type Refusal = RelationalStorePhysicalRefusal;
type Stage = RelationalStorePhysicalStage;

/// How a home reaches its Store: production media admission and the
/// production recovery entry. Certification may substitute a fault-scheduled
/// media admission or a recovery that parks at a process yieldpoint; every
/// other step is unchanged.
pub(crate) struct PhysicalOpening {
    media: FilesystemMediaAdmission,
    recovery_yieldpoint: Option<PhysicalRecoveryProcessYieldpoint>,
}

impl PhysicalOpening {
    pub(crate) fn production() -> Self {
        Self {
            media: FilesystemMediaAdmission::production(
                FilesystemAccessPosture::CoordinatedServiceAccount,
            ),
            recovery_yieldpoint: None,
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn with_media(mut self, media: FilesystemMediaAdmission) -> Self {
        self.media = media;
        self
    }

    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn with_recovery_yieldpoint(
        mut self,
        yieldpoint: PhysicalRecoveryProcessYieldpoint,
    ) -> Self {
        self.recovery_yieldpoint = Some(yieldpoint);
        self
    }
}

/// Create a new, empty Store at `root`.
pub(crate) fn initialize(
    root: &Path,
    opening: PhysicalOpening,
) -> Result<ServingPhysicalRuntime, Refusal> {
    let media = admit_media(root, opening.media)?;
    let durability = admit_durability(&media)?;
    let request = PhysicalRecordInitialization::new(
        profile::record_format(),
        profile::placement(),
        profile::access(),
        durability,
    )
    .with_journal_registry(crate::journal_registry::registry());
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
pub(crate) fn recover(
    root: &Path,
    opening: PhysicalOpening,
) -> Result<ServingPhysicalRuntime, Refusal> {
    let (held_media, custody) = recover_custody(root, opening.recovery_yieldpoint.clone())?;
    if let Some(yieldpoint) = &opening.recovery_yieldpoint {
        if yieldpoint
            .pause_after(
                worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ServingHandoff,
            )
            .is_interrupted()
        {
            return Err(Refusal::new(Stage::Recovery, "serving handoff interrupted"));
        }
    }
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let runtime = PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let media = match runtime
        .try_admit_recovered_filesystem_media(held_media, opening.media)
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        other => {
            return Err(Refusal::new(
                Stage::MediaAdmission,
                unfinished_outcome(&other),
            ))
        }
    };
    let durability = admit_durability(&media)?;
    let open = PhysicalRecordOpen::new(profile::record_format(), profile::access(), durability)
        .with_journal_registry(crate::journal_registry::registry());
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

fn recover_custody(
    root: &Path,
    yieldpoint: Option<PhysicalRecoveryProcessYieldpoint>,
) -> Result<
    (
        worth_store::physical_runtime::AdmittedRecoveryFilesystemMedia,
        Option<RecoveredPhysicalCheckpointCustody>,
    ),
    Refusal,
> {
    let limits = PhysicalRecoveryLimits::admit(profile::recovery_limits())
        .map_err(|denial| Refusal::new(Stage::RecoveryAdmission, format!("{denial:?}")))?;
    let configuration = profile::recovery_configuration();
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .map_err(recovery_admission_refusal)?;
    let backend = authority.qualified_backend_profile().clone();
    let request = PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    );
    let outcome = match yieldpoint {
        None => WorthStoreRecovery::recover(request),
        Some(yieldpoint) => {
            WorthStoreRecovery::recover_with_process_yieldpoint(request, yieldpoint)
        }
    };
    match outcome {
        PhysicalRecoveryOutcome::Recovered(handoff) => handoff
            .into_core()
            .into_serving_parts()
            .map(|(media, custody)| (media, Some(custody)))
            .ok_or_else(|| Refusal::new(Stage::Recovery, "selected custody missing")),
        other => Err(recovery_refusal::classify(other)),
    }
}

fn admit_media(
    root: &Path,
    media: FilesystemMediaAdmission,
) -> Result<MediaOwnedPhysicalRuntime, Refusal> {
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let runtime = PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    match runtime.try_admit_filesystem_media(media).into_raw() {
        TransitionOutcome::Success(media) => Ok(media),
        TransitionOutcome::Denied(denial) => {
            let reason = format!("{:?}", denial.reason());
            denial.into_runtime().close();
            Err(Refusal::new(Stage::MediaAdmission, reason))
        }
        TransitionOutcome::Deferred(deferred) => {
            let detail = format!("{:?}", deferred.reason());
            deferred.into_runtime().close();
            let mut refusal = Refusal::new(Stage::MediaAdmission, detail);
            refusal.kind = RelationalStorePhysicalRefusalKind::MutationOwnerContended;
            Err(refusal)
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

fn recovery_admission_refusal(error: PhysicalRecoveryPlatformAdmissionError) -> Refusal {
    let mut refusal = Refusal::new(Stage::RecoveryAdmission, format!("{error:?}"));
    if let PhysicalRecoveryPlatformAdmissionError::BackendQualification(cause) = error {
        use worth_store::physical_runtime::RecoveryFilesystemQualificationError as Cause;
        refusal.absent = matches!(cause, Cause::ExistingStoreRequired | Cause::RootUnavailable);
        if cause == Cause::OwnershipContended {
            refusal.kind = RelationalStorePhysicalRefusalKind::MutationOwnerContended;
        }
    }
    refusal
}

pub(crate) fn open(
    root: &Path,
    opening: PhysicalOpening,
) -> Result<ServingPhysicalRuntime, Refusal> {
    std::fs::create_dir_all(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, error.to_string()))?;
    match recover(
        root,
        PhysicalOpening {
            media: opening.media.clone(),
            recovery_yieldpoint: opening.recovery_yieldpoint.clone(),
        },
    ) {
        Ok(serving) => Ok(serving),
        Err(refusal) if refusal.absent => initialize(root, opening),
        Err(refusal) => Err(refusal),
    }
}
