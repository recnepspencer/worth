mod construction;
mod durability_bootstrap;
mod executor;
mod lifecycle;
mod parts;
mod residency_owner;
mod scheduler_admission;
mod signal_owner;
mod termination;
mod work_lifecycle;
mod work_runtime;

pub(in crate::physical_runtime) use construction::PhysicalStoreInstanceFoundation;
pub(in crate::physical_runtime) use durability_bootstrap::reopen_durability_basis;
pub use durability_bootstrap::PhysicalDurabilityStateReopenFailure;
pub(in crate::physical_runtime) use executor::PhysicalWorkExecutor;
#[cfg(feature = "certification-test-authority")]
pub use executor::{
    CertificationPhysicalExecutionCheckpoint, CertificationPhysicalExecutionPauseGate,
};
pub(in crate::physical_runtime) use parts::PhysicalStoreInstanceParts;
pub(in crate::physical_runtime) use residency_owner::PhysicalResidencyOwner;
pub(in crate::physical_runtime) use scheduler_admission::PhysicalScrubSchedulerAdmissionDenial;
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use scheduler_admission::PhysicalWalReclamationSchedulerAdmissionDenial;
pub(in crate::physical_runtime) use scheduler_admission::{
    PhysicalSchedulerAdmissionOwner, RecordSchedulerReservationDenial,
};
#[cfg(feature = "certification-test-authority")]
pub use signal_owner::CertificationPhysicalSignalPauseGate;
pub(in crate::physical_runtime) use signal_owner::{
    PhysicalSignalAdmissionStatus, PhysicalWorkSignalOwner,
};
pub use signal_owner::{
    PhysicalSignalClockObservation, PhysicalSignalClockObservationFailure,
    PhysicalSignalConstructionFailure, PhysicalSignalDeltaApplicationFailure,
    PhysicalSignalObservation, PhysicalSignalRuntimeIdentity, PhysicalSignalShutdownOutcome,
};
#[cfg(feature = "certification-test-authority")]
pub use termination::CertificationPhysicalClosePauseGate;
pub(in crate::physical_runtime) use termination::PhysicalStoreCloseProgressOwner;
pub use termination::{
    PhysicalStoreAbortOutcome, PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome,
    PhysicalStoreClosePhase, PhysicalStoreClosePlan,
};
pub(in crate::physical_runtime) use work_lifecycle::PhysicalWorkLifecycle;
pub(in crate::physical_runtime) use work_runtime::PhysicalExecutionCall;
pub use work_runtime::PhysicalWorkExecution;
pub(in crate::physical_runtime) use work_runtime::{
    PhysicalProjectionFailureCapability, PhysicalStoreWorkRuntime,
};
