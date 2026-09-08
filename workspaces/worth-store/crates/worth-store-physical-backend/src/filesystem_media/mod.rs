mod admission;
mod admission_operation_contracts;
mod admitted_namespace;
mod allocation;
mod artifact_mutation_coordinator;
mod artifact_tree;
mod artifact_tree_effects;
mod capability_profile;
mod capability_qualification;
#[cfg(feature = "certification-test-authority")]
mod certification_confinement_effects;
#[cfg(feature = "certification-test-authority")]
mod certification_fault_authority;
mod counter_observer;
mod counter_snapshot;
mod directory_handle;
mod directory_listing;
mod directory_synchronization;
mod durability_admission;
mod durable_deletion;
mod failure_context;
mod fault_activation;
mod fault_interposition;
mod fault_schedule;
#[cfg(any(test, feature = "certification-test-authority"))]
mod fault_schedule_validation;
mod file_handle;
mod file_mutation_sequence;
mod file_synchronization;
mod handle_accounting;
mod logical_length;
mod media_owner;
mod metadata;
mod mutation_lock_file;
mod mutation_owner_publication;
mod mutation_ownership;
mod named_file_identity;
mod namespace_admission;
mod namespace_confinement;
mod namespace_identity_admission;
mod namespace_publication;
mod namespace_publication_state;
mod namespace_root_inventory;
mod operation_context;
mod operation_contract;
mod operation_counters;
mod operation_role_metric;
mod outcome;
mod owner_admission_effect;
mod owner_local_identity;
mod pause_gate;
mod positioned_io;
mod positioned_read;
mod positioned_transfer;
mod profile_candidate_consistency;
mod profile_observation;
mod publication_summary;
mod qualification_basis;
mod qualification_basis_drift;
mod qualification_outcome;
mod qualification_report;
mod qualification_request;
#[cfg(any(test, feature = "certification-test-authority"))]
mod qualification_transaction;
mod qualified_capabilities;
#[cfg(feature = "recovery-runtime-owner")]
pub(crate) mod recovery_qualification;
mod staged_namespace_write;
mod synchronization;
mod transfer;

pub use admission::{AdmittedFilesystemMedia, QualifiedFilesystemMedia};
pub use admitted_namespace::AdmittedStoreNamespace;
pub use allocation::{
    AllocationRequest, MediaAllocationMode, MediaAllocationObservation, MediaAllocationOutcome,
    MediaAllocationResult, MediaPhysicalAllocationPosture,
};
pub use artifact_tree::{
    ArtifactAppendOutcome, ArtifactAppendRange, ArtifactNewWriteOutcome, ArtifactNewWriteRange,
    ArtifactRangeReadOutcome, ArtifactRangeWriteDurability,
    ArtifactRangeWriteDurabilityRequirement, ArtifactRangeWriteOutcome, ArtifactTreeAccessLimit,
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
    ArtifactTreeMedia, ArtifactTreeNewFile, ArtifactTreePathDenial, ArtifactTreePublicationEffect,
    ArtifactTreePublicationEffectOutcome, ArtifactTreeReplacement, CompletedArtifactAppend,
    CompletedArtifactMetadataRead, CompletedArtifactNewWrite, CompletedArtifactRangeRead,
    CompletedArtifactRangeWrite, CompletedArtifactTreePublicationEffect,
    CompletedScheduledArtifactAppend, CompletedScheduledArtifactMetadataRead,
    CompletedScheduledArtifactNewWrite, CompletedScheduledArtifactRangeRead,
    CompletedScheduledArtifactRangeWrite, CompletedScheduledArtifactTreePublicationEffect,
    IndeterminateArtifactAppend, IndeterminateArtifactNewWrite, IndeterminateArtifactRangeWrite,
    IndeterminateArtifactTreePublicationEffect, InspectionSourceVersion,
    ObservedArtifactInspectionRead, ScheduledArtifactAppendOutcome,
    ScheduledArtifactInspectionReadOutcome, ScheduledArtifactMetadataReadOutcome,
    ScheduledArtifactNewWriteOutcome, ScheduledArtifactRangeReadOutcome,
    ScheduledArtifactRangeWriteOutcome, ScheduledArtifactTreePublicationEffectOutcome,
};
pub use capability_profile::{
    CapabilityProfileError, CapabilitySupport, FilesystemBackendProfile, FilesystemLocation,
    MediaCapability, MediaCapabilityObservation,
};
pub use capability_qualification::MediaCapabilityQualificationOutcome;
#[cfg(feature = "certification-test-authority")]
pub use certification_fault_authority::{
    certification_media_fault_authority, CertificationMediaFaultAuthority,
};
pub use counter_observer::{MediaCounterObserver, MediaCounterOverflowPolicy};
pub use counter_snapshot::{MediaCounterSnapshot, MediaCounterTerminal};
pub use directory_handle::{ArtifactFamilyDirectory, NamespaceDirectoryHandle, StagingDirectory};
pub use directory_listing::{
    NamespaceDirectoryListing, NamespaceDirectoryListingResult, NamespaceEntry,
    NamespaceEntryBatch, NamespaceEntryBatchOutcome, NamespaceEntryBatchResult,
    MAX_DIRECTORY_BATCH_ENTRIES,
};
pub use durable_deletion::{
    DurableDeletion, DurableDeletionOutcome, IndeterminateNamespaceDeletion,
    NamespaceDeletionOutcome, VisibleNamespaceDeletion,
};
pub use failure_context::{
    MediaCausalBoundary, MediaFailureContext, MediaOsCode, MediaOsCodeFamily, MediaPathRole,
};
#[cfg(any(test, feature = "certification-test-authority"))]
pub use fault_activation::{CertificationMediaFaultActivation, MediaFaultActivationDenial};
pub use fault_schedule::{
    MediaFaultDirective, MediaFaultRule, MediaFaultSchedule, MediaFaultScheduleDenial,
};
#[cfg(all(
    feature = "certification-test-authority",
    feature = "recovery-runtime-owner"
))]
pub(crate) use file_handle::CertificationRetainedMediaFileHandle;
pub use file_handle::{
    MutableFileAccess, NamespaceFileHandle, NamespaceFileOpenKind, NamespaceFileOpenOutcome,
    NamespaceFileOpenResult, ReadOnlyFileAccess,
};
pub use logical_length::TruncateRequest;
pub use media_owner::{
    FilesystemMediaAdmissionAuthority, FilesystemMediaOwner, FilesystemMediaOwnerAdmissionDenial,
};
pub use metadata::{
    MediaAllocatedBytes, MediaFileType, MediaMetadata, MediaMetadataOutcome, MediaMetadataResult,
};
pub use mutation_ownership::{
    MutationOwnerObservation, MutationOwnershipAttempt, MutationOwnershipDenial,
    MutationOwnershipLease, OwnershipReleaseOutcome,
};
#[cfg(feature = "certification-test-authority")]
pub use namespace_confinement::CertificationConfinementEffect;
pub use namespace_confinement::{
    NamespaceConfinementDenial, NamespaceConfinementDenialKind, NamespacePublicationTarget,
    NamespaceRelativePath, StagedNamespacePath,
};
pub use namespace_publication::{
    AtomicReplacementOutcome, CompletedAtomicReplacement, CompletedStagedNamespaceWrite,
    DurableNamespacePublicationOutcome, DurablyPublishedNamespaceFile,
    IndeterminateNamespacePublication, NamespacePublicationStage, StagedNamespaceFile,
    StagedNamespaceFileOutcome, StagedNamespaceSynchronizationOutcome, StagedNamespaceWriteOutcome,
    SynchronizedStagedNamespaceFile,
};
pub use operation_context::MediaOperationContext;
use operation_context::{MediaOperationCoordinates, MediaOperationIdentityBinding};
pub use operation_contract::{
    MediaCallAudience, MediaCapabilityRequirement, MediaCounterClass, MediaFaultControlAudience,
    MediaHandleRequirement, MediaObservationAudience, MediaOperationContract, MediaOperationRole,
    MediaPartialEffect, MediaRetryRule, MediaSynchronizationMeaning, MediaTransferCardinality,
};
pub use outcome::{
    CompletedMediaEffect, MediaAttemptedEffect, MediaEffectStatus, MediaEstablishedBoundary,
    MediaOperationFailure, MediaOperationFailureKind, MediaOperationOutcome, MediaOperationResult,
    MediaRetryPosture, PositionedReadOutcome, PositionedReadResult,
};
pub use owner_local_identity::{
    MediaHandleIdentity, MediaOperationIdentity, MediaOwnerIdentity, MediaQualificationIdentity,
};
pub use pause_gate::MediaPauseGate;
pub use positioned_transfer::{AppendRequest, PositionedReadRequest, PositionedWriteRequest};
pub use profile_observation::filesystem_media_build_identity;
pub use publication_summary::{NamespacePublicationSummary, PublicationWriteSummary};
pub use qualification_basis::RootProfileQualificationBasis;
pub use qualification_basis_drift::MediaQualificationBasisDrift;
pub use qualification_outcome::{
    MediaQualificationDeferred, MediaQualificationDenial, MediaQualificationFailure,
    MediaQualificationPostOwnershipCause, MediaQualificationRebindRequired,
    MediaQualificationStale,
};
pub use qualification_report::RootProfileQualificationReport;
pub use qualification_request::{
    FilesystemAccessContract, FilesystemAccessPosture, FilesystemQualificationMode,
    FilesystemQualificationRequest,
};
pub use qualified_capabilities::{
    AllocationLengthPosture, DataSyncMetadataPosture, MappedDurabilityPosture,
    MappedTruncationPosture, MediaCapabilityScope, QualifiedBaseMediaCapabilities,
    QualifiedDataSyncCapability, QualifiedDirectIoCapability, QualifiedMediaCapabilities,
    QualifiedMmapCapability, QualifiedPreallocationCapability, QualifiedSparseAllocationCapability,
};
pub use synchronization::{
    DirectoryPublicationSynchronization, DirectoryPublicationSynchronizationOutcome,
    FileDataSynchronization, FileDataSynchronizationOutcome, FileStateSynchronization,
    FileStateSynchronizationOutcome, RootParentPublicationSynchronization,
    RootParentPublicationSynchronizationOutcome, StoreRootPublicationSynchronization,
    StoreRootPublicationSynchronizationOutcome,
};
pub use transfer::{
    CompletedMediaTransfer, MediaTransferPosition, MediaTransferProgress, MediaTransferShapeError,
    PartialMediaTransfer,
};

/// Narrow composition seam used by the Store-owned runtime facade.
///
/// The backend owns filesystem qualification and the resulting media owner;
/// it does not own or promote the Store runtime phase that consumes this
/// outcome.
#[doc(hidden)]
#[cfg(feature = "store-runtime-owner")]
pub fn qualify_filesystem_media(
    request: FilesystemQualificationRequest,
) -> AdmittedFilesystemMedia {
    FilesystemMediaOwner::qualify(request)
}

#[cfg(test)]
mod tests;
