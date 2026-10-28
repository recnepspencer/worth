mod admission;
mod admission_outcome;
mod observation;
#[cfg(feature = "recovery-runtime-owner")]
mod recovery_cleanup;
mod runtime;
mod shutdown;

pub use admission::FilesystemMediaAdmission;
pub use admission_outcome::{
    MediaAdmissionDeferred, MediaAdmissionDenial, MediaAdmissionInspectionCause,
    MediaAdmissionInspectionRequired, MediaAdmissionOutcome, MediaAdmissionRebindRequired,
    MediaAdmissionStale,
};
pub use observation::{
    MediaOwnedObservationPhase, PhysicalMediaObservation, PhysicalMediaObserver,
    RecordServingObservationPhase,
};
#[cfg(feature = "recovery-runtime-owner")]
pub use recovery_cleanup::{
    CompletedRecoveryCleanupPhysicalRemoval, DeniedRecoveryCleanupPhysicalRemoval,
    IndeterminateRecoveryCleanupPhysicalRemoval, RecoveryCleanupArtifactRevalidationDenial,
    RecoveryCleanupArtifactRevalidationProgress, RecoveryCleanupRemovalDenialCause,
    RecoveryCleanupRemovalOutcome,
};
pub use runtime::MediaOwnedPhysicalRuntime;
pub use shutdown::MediaShutdownOutcome;

pub(super) use admission::try_admit;

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use admission::admit_recovered;
