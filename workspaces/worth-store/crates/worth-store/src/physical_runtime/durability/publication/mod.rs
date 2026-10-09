mod current_root_owner;
#[cfg(feature = "certification-test-authority")]
pub use current_root_owner::{
    CertificationReadRootCapturePauseGate, CertificationReadRootCaptureStage,
    CertificationReleaseHeadObservation,
};
pub use current_root_owner::{ReleaseCertificateCapacityDenial, SelectedReleaseHeadDenial};
mod failure;
mod identity;
mod maintenance;
mod namespace_durability;
mod preparation;
mod replacement;
mod retained_root;
mod terminal_head_retirement_authority;
mod work_port;
pub(in crate::physical_runtime) use current_root_owner::{
    AdmittedTerminalHeadRetirement, CheckpointAttestedTerminalHead, PublicationStateLockHeld,
    TerminalHeadAttestationDenial, TerminalHeadPublicationExcluded,
    TerminalHeadRetirementAdmissionDenial,
};
pub(in crate::physical_runtime) use maintenance::{
    publish_manifest_residue_candidate, publish_retirement_candidate,
    NamespaceDurableManifestResidueRoot, NamespaceDurableRetirementRoot,
};
#[cfg(feature = "certification-test-authority")]
pub(in crate::physical_runtime) use maintenance::{
    publish_tier_epoch_candidate, NamespaceDurableTierEpochRoot,
};
pub(in crate::physical_runtime) use terminal_head_retirement_authority::TerminalHeadRetirementAuthority;

#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use current_root_owner::RecoveredReleaseLedgerDenial;
pub(in crate::physical_runtime) use current_root_owner::{
    AdmittedFailedIngestDrop, AdmittedManifestResidueRetirement, AdmittedReleasedGenerationDrop,
    CheckpointCertificateFrame, CheckpointCustodyCandidate, CheckpointCustodyDenial,
    CheckpointCustodyOrigin, CleanReopenCheckpointCustody, ManifestResidueDisplacement,
    ManifestResidueProof, PhysicalBlobReclaimAdmissionDenial, PhysicalBlobSessionClaim,
    PhysicalBlobSessionClaimDenial, PhysicalBlobTerminalAdmissionDenial, PhysicalCurrentRootOwner,
    PhysicalReclaimAttempt, PhysicalReconciledReclaimDescriptorFate,
    PreparedRecoveredCheckpointCustody, ReleaseCertificateCapacityLease, ReleaseHeadCapacityCharge,
    ReleasedDropSourceCaptureDenial, SelectedCheckpointCustodySnapshot, SelectedOriginalDropProof,
    SelectedReleaseHeadBasis, ServingCheckpointCustody,
};
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
