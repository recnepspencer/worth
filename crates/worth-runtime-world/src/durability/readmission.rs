//! Owner-frame readmission and cross-journal consistency. These are recovered
//! descriptions, never Native performed capabilities or live product heads.
use super::{DecodedWorldRecord, WorldCheckpointImage, WorldDurableMediumDenial, WorldRecord};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableProductUnpublishedOwnerEffectsIdentity, WorldDurableJournal,
};
mod disposition;
mod journal;
mod occurrence;
mod occurrence_admission;
pub(crate) use disposition::AdmittedWorldDisposition;
mod open;
pub(crate) use occurrence::{AdmittedWorldOccurrence, AdmittedWorldOccurrenceLocation};
pub(crate) use open::AdmittedWorldOpenAttempt;

#[derive(Debug)]
pub enum WorldReadmissionDenial {
    JournalSourceMismatch,
    OccurrenceEvidenceMismatch,
    HistoryAdmission(crate::history::CompositeHistoryCatalogDenial),
    HistoryRetention(crate::branch::RuntimeWorldBootstrapNoEffectCause),
    NativeCustody {
        cause: Box<WorldReadmissionDenial>,
        custody: Box<super::WorldNativeReadmissionCustody>,
    },
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
    ConflictingOpenAttempt,
    ConflictingDisposition,
    MissingPredecessor,
    BranchLineage,
    FutureIncarnation,
    NativeReadmissionRequired,
    RootPreparation(Box<super::RefusedWorldRootReadmission>),
    SignalInitialRoot(worth_signal::facade::branch::SignalInitialEmptyRootRefusal),
    BridgeBaseline(worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal),
    NativeRecorded(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    RootAdmission(crate::branch::RuntimeWorldBootstrapNoEffectCause),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum RecoveredCustodyDisposition {
    Released(Option<super::WorldRelationalReleaseEvidence>),
    Continued,
}

pub(crate) type WorldRecoveredJournalState = WorldCheckpointImage;
