//! Stable cascade admission failures; the publication context carries exact records.
use serde::{Deserialize, Serialize};

/// Which writer of one coupled candidate created, or tried to destroy, a fresh
/// record. In-memory and wire refusal detail only; never stored in history.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CoupledFreshWriter {
    /// The authored writer: the authored prefix and every later authored stage.
    Authored,
    /// The producer part at this index in the producer roster.
    Producer(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthoredRetirementRefusal {
    OrdinaryMetadataWrite,
    CurrentOwnershipStorageMissing,
    OwnedPublicationRequiresCascade {
        publication: crate::history::data::CommitId,
        owner: crate::identity::data::EntityId,
    },
    PublicationOwnerUnavailable {
        publication: crate::history::data::CommitId,
        owner: crate::identity::data::EntityId,
    },
    ForeignFreshCoupledRecord {
        creator: CoupledFreshWriter,
        writer: CoupledFreshWriter,
        owner: crate::identity::data::EntityId,
        source: crate::identity::data::EntityId,
    },
    MissingCoupledOwner,
    ExtraCoupledOwner,
    ForeignCoupledOwner,
    RepeatedCoupledOwner,
    ForeignCoupledContinuation,
    StaleCoupledContinuation,
    CoupledContinuationRequiresOwnedRoute,
    /// An explicit retirement, restoration or selection writer runs exactly once.
    CoupledContinuationRequiresEditSource,
    /// Only the new atomic continuation entry refuses; existing successor delivery stays available.
    CoupledContinuationProvenancePostponed {
        issue: u16,
    },
    MissingMember,
    MissingOwnedMember,
    ExtraOwnedMember,
    ForeignOwnedMember,
    RepeatedOwnedMember,
    DanglingOwnedMember,
    MissingPublication,
    ExtraEntity,
    UnrelatedPublication,
    RepeatedTarget,
    DifferentSourcePublication,
    InvalidAuthoredAnchor,
    CorruptPublication,
    CorruptCurrentOwnership {
        publication: crate::history::data::CommitId,
    },
    IncompleteRestoration,
    HistoricalSelectionBasisChanged,
    HistoricalSelectionRouteMissing,
    HistoricalSelectionNotOwned {
        publication: crate::history::data::CommitId,
    },
    CoupledSourceNotLive,
    /// An authored stage appended out of the Native-admitted stage roster order.
    UnadmittedAuthoredStage,
    /// The assembly completed before every admitted authored stage applied.
    MissingAuthoredStage,
}
