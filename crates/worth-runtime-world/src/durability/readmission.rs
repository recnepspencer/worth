//! Owner-frame readmission and cross-journal consistency. These are recovered
//! descriptions, never Native performed capabilities or live product heads.
use super::{DecodedWorldRecord, WorldCheckpointImage, WorldDurableMediumDenial, WorldRecord};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableProductUnpublishedOwnerEffectsIdentity, WorldDurableJournal,
};
mod collected_occurrence;
mod creation;
mod creation_admission;
mod disposition;
pub(crate) use collected_occurrence::CollectedWorldOccurrence;
pub(crate) use creation::{
    AdmittedWorldCreationIntent, AdmittedWorldCreationRelease, RecordedWorldBranchCreated,
};
mod dependency_order;
mod journal;
mod occurrence;
mod occurrence_admission;
mod reference_history;
pub(crate) use dependency_order::CollectedWorldHistoryStep;
pub(crate) use disposition::{AdmittedWorldDisposition, PreparedWorldReleasedDisposition};
mod open;
pub(crate) use occurrence::{AdmittedWorldOccurrence, AdmittedWorldOccurrenceLocation};
pub(crate) use open::AdmittedWorldOpenAttempt;

#[derive(Debug)]
pub enum WorldReadmissionDenial {
    AuthoredAttachment(Box<crate::lifecycle::RuntimeWorldAuthoredSignalAttachmentDenial>),
    JournalSourceMismatch,
    OccurrenceEvidenceMismatch,
    HistoryAdmission(crate::history::CompositeHistoryCatalogDenial),
    HistoryRetention(crate::branch::RuntimeWorldBootstrapNoEffectCause),
    NativeCustody {
        cause: Box<WorldReadmissionDenial>,
        custody: Box<super::WorldNativeReadmissionCustody>,
    },
    NativeForkRetirement(worth_relational::facade::branch::RelationalBranchBasisRetirementDenial),
    NativeForkDiscovery(worth_relational::facade::durability::RelationalForkReadmissionCause),
    NativeFork(Box<worth_relational::facade::durability::RefusedRelationalForkReadmission>),
    NativeDiscovery(worth_relational::facade::durability::RelationalAssociatedEffectRefusal),
    NativeComponent(Box<worth_relational::facade::durability::RefusedRelationalRecordedComponent>),
    NativePublication(
        Box<worth_relational::facade::durability::RefusedRelationalPublicationReadmission>,
    ),
    NativeSuccessor(worth_relational::facade::durability::RelationalRecordedSuccessorRefusal),
    NativeSettlementRequired,
    OrphanNativeAssociation,
    DuplicateNativeEffect,
    MalformedNativeAssociation,
    DispositionEvidenceMismatch,
    ProgramRevisionMismatch,
    OccurrenceReadmissionRequired,
    RecoveryCapacity,
    ExecutionInstallation(crate::lifecycle::RuntimeWorldBuildDenial),
    Medium(WorldDurableMediumDenial),
    IdentityExhaustion(crate::identity::RuntimeWorldIdentityExhaustion),
    WrongJournal,
    ForeignOwner,
    DuplicateIdentity,
    MissingOpenAttempt,
    MissingCreationIntent,
    ConflictingCreationIntent,
    CreationEvidenceMismatch,
    ConflictingOpenAttempt,
    ConflictingDisposition,
    MissingPredecessor,
    BranchLineage,
    FutureIncarnation,
    NativeReadmissionRequired,
    RootPreparation(Box<super::RefusedWorldRootReadmission>),
    AuthoredSignalRequired,
    SignalDiscovery(Box<worth_signal::facade::durability::SignalOwnerReadmissionDenial>),
    UncleanRecoveryUnsupported(worth_foundational::facade::UncleanRecoveryUnsupported),
    SignalSource(Box<worth_signal::facade::durability::SignalSourcePointReadmissionDenial>),
    BridgeBaseline(worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal),
    NativeRecorded(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    RootAdmission(crate::branch::RuntimeWorldBootstrapNoEffectCause),
    FreshWorldSourceNotEmpty,
    NativeFreshSource(worth_relational::facade::durability::RelationalRecoveryStop),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum RecoveredCustodyDisposition {
    Released(Option<super::WorldRelationalReleaseEvidence>),
    Continued,
}

pub(crate) type WorldRecoveredJournalState = WorldCheckpointImage;
