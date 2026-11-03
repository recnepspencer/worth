//! The physical Store under a application home: created once, and after that
//! only ever recovered. A home never opens a Store that recovery has not
//! admitted, and never initializes over existing data.

mod media_admission;
mod profile;
mod recovered_serving;
mod recovery_refusal;

pub(crate) use profile::placement;
#[cfg(feature = "certification-test-authority")]
pub use profile::ApplicationHomeCertificationProfile;

use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, MediaOwnedPhysicalRuntime,
    PhysicalRecordInitialization, PhysicalRecoveryProcessYieldpoint, PhysicalRuntimeAdmission,
    PhysicalStore, ServingPhysicalRuntime,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlatformAdmissionError, PhysicalRecoveryPlatformAuthority, WorthStoreRecovery,
};

/// Which physical step refused, so a caller can tell an unusable location
/// from a Store that recovery would not admit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApplicationHomePhysicalStage {
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
pub struct ApplicationHomePhysicalRefusal {
    stage: ApplicationHomePhysicalStage,
    detail: String,
    kind: ApplicationHomePhysicalRefusalKind,
    recovery_effects: Option<u64>,
}

impl ApplicationHomePhysicalRefusal {
    fn new(stage: ApplicationHomePhysicalStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
            kind: ApplicationHomePhysicalRefusalKind::PhysicalStep,
            recovery_effects: None,
        }
    }

    /// Recovery effects already performed when the physical refusal was decided.
    pub fn recovery_effects(&self) -> Option<u64> {
        self.recovery_effects
    }

    pub fn kind(&self) -> ApplicationHomePhysicalRefusalKind {
        self.kind
    }

    pub fn stage(&self) -> ApplicationHomePhysicalStage {
        self.stage
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplicationHomePhysicalRefusalKind {
    PhysicalStep,
    MutationOwnerContended,
    UnsupportedRootSchema { observed: u32 },
}

type Refusal = ApplicationHomePhysicalRefusal;
type Stage = ApplicationHomePhysicalStage;

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
#[cfg(test)]
pub(crate) fn initialize(
    root: &Path,
    opening: PhysicalOpening,
) -> Result<ServingPhysicalRuntime, Refusal> {
    initialize_owned(media_admission::admit_fresh(root, opening.media)?)
}

fn initialize_owned(media: MediaOwnedPhysicalRuntime) -> Result<ServingPhysicalRuntime, Refusal> {
    let durability = admit_durability(media.physical_durability_admission_basis())?;
    let request = PhysicalRecordInitialization::new(
        profile::record_format(),
        profile::placement(),
        profile::access(),
        durability,
    )
    .with_journal_registry(crate::registry::registry());
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
#[inline(never)]
fn recover(
    root: &Path,
    opening: PhysicalOpening,
    held: worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia,
) -> Result<ServingPhysicalRuntime, Refusal> {
    let request = prepare_recovery_request(root, held)?;
    let outcome = recover_physical(request, opening.recovery_yieldpoint.clone());
    recovered_serving::from_recovery_outcome(root, opening, outcome)
}

#[inline(never)]
fn recover_physical(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<PhysicalRecoveryProcessYieldpoint>,
) -> PhysicalRecoveryOutcome {
    match yieldpoint {
        None => WorthStoreRecovery::recover(request),
        Some(yieldpoint) => {
            WorthStoreRecovery::recover_with_process_yieldpoint(request, yieldpoint)
        }
    }
}

#[inline(never)]
fn prepare_recovery_request(
    root: &Path,
    media: worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia,
) -> Result<PhysicalRecoveryOpenRequest, Refusal> {
    let limits = PhysicalRecoveryLimits::admit(profile::recovery_limits())
        .map_err(|denial| Refusal::new(Stage::RecoveryAdmission, format!("{denial:?}")))?;
    let configuration = profile::recovery_configuration();
    let authority = PhysicalRecoveryPlatformAuthority::from_held_media(
        root.to_owned(),
        configuration.clone(),
        limits,
        media,
    )
    .map_err(recovery_admission_refusal)?;
    let backend = authority.qualified_backend_profile().clone();
    Ok(PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    ))
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
    basis: Result<
        worth_store::physical_runtime::PhysicalDurabilityAdmissionBasis,
        worth_store::physical_runtime::BackendCapabilityAdmissionDenial,
    >,
) -> Result<worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy, Refusal> {
    let basis =
        basis.map_err(|error| Refusal::new(Stage::DurabilityAdmission, format!("{error:?}")))?;
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
        if cause == Cause::OwnershipContended {
            refusal.kind = ApplicationHomePhysicalRefusalKind::MutationOwnerContended;
        }
    }
    refusal
}

pub(crate) struct OpenedPhysicalHome {
    pub(crate) serving: ServingPhysicalRuntime,
    pub(crate) origin: worth_store::physical_runtime::RecoveryStorePresence,
}

pub(crate) fn open(root: &Path, opening: PhysicalOpening) -> Result<OpenedPhysicalHome, Refusal> {
    std::fs::create_dir_all(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, error.to_string()))?;
    let held = opening.media.qualify_home(root).map_err(|error| {
        recovery_admission_refusal(
            PhysicalRecoveryPlatformAdmissionError::BackendQualification(error),
        )
    })?;
    match held
        .store_presence()
        .map_err(|error| Refusal::new(Stage::RecoveryAdmission, format!("{error:?}")))?
    {
        worth_store::physical_runtime::RecoveryStorePresence::ProvenAbsent => {
            let serving = initialize_owned(media_admission::admit_for_initialization(
                root,
                held,
                opening.media,
            )?)?;
            Ok(OpenedPhysicalHome {
                serving,
                origin: worth_store::physical_runtime::RecoveryStorePresence::ProvenAbsent,
            })
        }
        worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts => {
            Ok(OpenedPhysicalHome {
                serving: recover(root, opening, held)?,
                origin: worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts,
            })
        }
    }
}
