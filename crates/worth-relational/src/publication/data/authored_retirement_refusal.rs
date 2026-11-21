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
    /// The successor does not re-patch a field, or re-create a record, the
    /// chain's earlier round wrote.
    UncoveredWrite,
    /// The successor deletes a record its own chain created.
    FreshDelete,
    /// The successor read a record the chain's earlier round wrote (Query).
    PriorEffectRead,
    /// The successor has an effect a fold cannot lower in place: a deletion,
    /// a relation change, a new record or content.
    StructuralEffect,
    /// Another writer changed a record of the chain after its earlier round.
    IntervenedOutput,
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
