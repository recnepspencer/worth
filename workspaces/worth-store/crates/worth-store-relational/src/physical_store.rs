//! The physical Store under a Relational home: created once, and after that
//! only ever recovered. A home never opens a Store that recovery has not
//! admitted, and never initializes over existing data.

mod media_admission;
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
    PhysicalRuntimeAdmission, PhysicalStore, RecoveredPhysicalServingHandoff,
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
    kind: RelationalStorePhysicalRefusalKind,
    recovery_effects: Option<u64>,
}

impl RelationalStorePhysicalRefusal {
    fn new(stage: RelationalStorePhysicalStage, detail: impl Into<String>) -> Self {
        Self {
            stage,
            detail: detail.into(),
            kind: RelationalStorePhysicalRefusalKind::PhysicalStep,
            recovery_effects: None,
        }
    }

    /// Recovery effects already performed when the physical refusal was decided.
    pub fn recovery_effects(&self) -> Option<u64> {
        self.recovery_effects
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
fn recover(
    root: &Path,
    opening: PhysicalOpening,
    held: worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia,
) -> Result<ServingPhysicalRuntime, Refusal> {
    let handoff = recover_custody(root, opening.recovery_yieldpoint.clone(), held)?;
    if let Some(yieldpoint) = &opening.recovery_yieldpoint
        && yieldpoint
            .pause_after(
                worth_store::physical_runtime::PhysicalRecoveryYieldpointStage::ServingHandoff,
            )
            .is_interrupted()
    {
        return Err(Refusal::new(Stage::Recovery, "serving handoff interrupted"));
    }
    let admission = PhysicalRuntimeAdmission::new(root)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let runtime = PhysicalStore::admit(admission)
        .map_err(|error| Refusal::new(Stage::RuntimeAdmission, format!("{error:?}")))?;
    let media = match handoff.admit_serving(runtime, opening.media).into_raw() {
        TransitionOutcome::Success(media) => media,
        other => {
            return Err(Refusal::new(
                Stage::MediaAdmission,
                unfinished_outcome(&other),
            ))
        }
    };
    let durability = admit_durability(media.physical_durability_admission_basis())?;
    let open = PhysicalRecordOpen::new(profile::record_format(), profile::access(), durability)
        .with_journal_registry(crate::journal_registry::registry());
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
    media: worth_store::physical_runtime::QualifiedRecoveryFilesystemMedia,
) -> Result<RecoveredPhysicalServingHandoff, Refusal> {
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
            .into_serving_handoff()
            .ok_or_else(|| Refusal::new(Stage::Recovery, "selected custody missing")),
        other => Err(recovery_refusal::classify(other)),
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
    let held = opening.media.qualify_home(root).map_err(|error| {
        recovery_admission_refusal(
            PhysicalRecoveryPlatformAdmissionError::BackendQualification(error),
        )
    })?;
    match held
        .store_presence()
        .map_err(|error| Refusal::new(Stage::RecoveryAdmission, format!("{error:?}")))?
    {
        worth_store::physical_runtime::RecoveryStorePresence::ProvenAbsent => initialize_owned(
            media_admission::admit_for_initialization(root, held, opening.media)?,
        ),
        worth_store::physical_runtime::RecoveryStorePresence::ExistingArtifacts => {
            recover(root, opening, held)
        }
    }
}
