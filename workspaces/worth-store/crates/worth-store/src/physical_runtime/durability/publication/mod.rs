mod current_root_owner;
#[cfg(feature = "certification-test-authority")]
pub use current_root_owner::{
    CertificationReadRootCapturePauseGate, CertificationReadRootCaptureStage,
};
mod failure;
mod identity;
mod namespace_durability;
mod preparation;
mod replacement;
mod retained_root;
mod work_port;

pub(in crate::physical_runtime) use current_root_owner::PhysicalCurrentRootOwner;
pub use current_root_owner::{
    CompletedPhysicalRootPublication, IndeterminatePhysicalCurrentRootAdvance,
    PhysicalCurrentRootAdvanceFailureCause, PhysicalCurrentRootAdvanceOutcome,
};

pub use failure::{
    IndeterminatePhysicalRootPublicationPreparation,
    PhysicalRootCandidateSynchronizationFailureCause, PhysicalRootCandidateWriteFailureCause,
    PhysicalRootCandidateWriteFailurePosture, PhysicalRootPublicationPreparationFailureCause,
    PhysicalRootPublicationPreparationNotStarted, PhysicalRootPublicationPreparationOutcome,
};
pub(in crate::physical_runtime) use failure::{
    PhysicalRootPublicationPreparationFailure, PhysicalRootPublicationPreparationNotStartedCause,
    RootCandidateSynchronizationFailure,
};
pub(in crate::physical_runtime) use identity::PhysicalRootPublicationIdentity;
pub use identity::PhysicalRootPublicationMemberIdentity;
pub(in crate::physical_runtime) use namespace_durability::synchronize_root_namespace;
pub use namespace_durability::{
    IndeterminatePhysicalRootNamespaceDurability, PhysicalRootNamespaceDurabilityFailureCause,
    PhysicalRootNamespaceDurabilityNotStarted, PhysicalRootNamespaceDurabilityOutcome,
};
pub use preparation::PhysicalRootPublicationTransitionDenial;
pub(in crate::physical_runtime) use preparation::{
    PhysicalRootPublicationTransition, PhysicalRootPublicationTransitionOwner,
};
pub(in crate::physical_runtime) use replacement::replace_root_candidate;
pub use replacement::{
    IndeterminatePhysicalRootReplacement, PhysicalRootReplacementFailureCause,
    PhysicalRootReplacementNotStarted, PhysicalRootReplacementOutcome,
};
pub use retained_root::RetainedPhysicalRoot;
pub use work_port::PhysicalRootPublicationWorkFailureCause;
pub(in crate::physical_runtime) use work_port::{
    PhysicalRootPublicationWorkFailure, PhysicalRootPublicationWorkPort,
};
