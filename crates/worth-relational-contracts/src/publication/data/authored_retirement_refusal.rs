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

/// Why one route's later producing round cannot publish under a single-input
/// origin yet (public #119 multi-input provenance). The refusal's affected
/// record names the route source or the exact record where known.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CoupledProvenancePostponement {
    /// A verified continuation round would change the candidate.
    ContinuedRoundEffect,
    /// The successor read a record the chain's earlier round wrote (Query), or
    /// names a record only a superseded round of its chain created (Native).
    PriorEffectRead,
    /// Another writer changed, after the chain's head, a record the chain
    /// changed or the successor round changes.
    IntervenedOutput,
    /// Another writer links to, names or assigns a role to a record the
    /// successor round no longer creates.
    RetiredReference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthoredRetirementRefusal {
    OrdinaryMetadataWrite,
    CurrentOwnershipStorageMissing,
    OwnedPublicationRequiresCascade {
        publication: crate::history::data::coordinates::CommitId,
        owner: crate::identity::data::EntityId,
    },
    PublicationOwnerUnavailable {
        publication: crate::history::data::coordinates::CommitId,
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
        reason: CoupledProvenancePostponement,
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
        publication: crate::history::data::coordinates::CommitId,
    },
    IncompleteRestoration,
    HistoricalSelectionBasisChanged,
    HistoricalSelectionRouteMissing,
    HistoricalSelectionNotOwned {
        publication: crate::history::data::coordinates::CommitId,
    },
    CoupledSourceNotLive,
    /// An authored stage appended out of the Native-admitted stage roster order.
    UnadmittedAuthoredStage,
    /// The assembly completed before every admitted authored stage applied.
    MissingAuthoredStage,
    /// Stage `stage` (0 is the prefix) retires `owner`; only the prefix retires (public #`issue`).
    LaterStageRetirementPostponed {
        issue: u16,
        stage: usize,
        owner: crate::identity::data::EntityId,
    },
}
