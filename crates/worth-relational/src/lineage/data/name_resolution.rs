//! Resolution vocabulary for held persistent names over the branch-local
//! derived name index.

use std::collections::BTreeSet;

use worth_foundational::facade::AggregateIdentity;

use crate::history::data::CommitId;
use crate::identity::data::{EntityId, LineageId, RelationId};

use super::naming::{PersistentNameIdentity, PersistentNameSubjectKind};

/// A held persistent name of any subject class, as the index keys it.
///
/// Entity and relation names are minted [`PersistentNameIdentity`] values;
/// group names are allocation-independent [`AggregateIdentity`] values. The
/// variant keeps the subject class explicit, so one subject's name can never
/// answer for another's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersistentNameKey {
    Entity(PersistentNameIdentity),
    Relation(PersistentNameIdentity),
    Group(AggregateIdentity),
}

impl PersistentNameKey {
    /// The fixed 32-byte canonical digest of the held name.
    pub const fn digest(&self) -> &[u8; 32] {
        match self {
            Self::Entity(name) | Self::Relation(name) => name.as_bytes(),
            Self::Group(group) => group.bytes(),
        }
    }

    pub const fn subject_kind(&self) -> PersistentNameSubjectKind {
        match self {
            Self::Entity(_) => PersistentNameSubjectKind::Entity,
            Self::Relation(_) => PersistentNameSubjectKind::Relation,
            Self::Group(_) => PersistentNameSubjectKind::Group,
        }
    }
}

/// The live record a persistent name designates at a resolution basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersistentNameTarget {
    Entity {
        name: PersistentNameIdentity,
        entity: EntityId,
        lineage: LineageId,
    },
    Relation {
        name: PersistentNameIdentity,
        relation: RelationId,
        lineage: LineageId,
    },
    /// A live group; its identity is both its name and its handle.
    Group { group: AggregateIdentity },
}

impl PersistentNameTarget {
    /// The persistent name this target is live under.
    pub const fn key(&self) -> PersistentNameKey {
        match self {
            Self::Entity { name, .. } => PersistentNameKey::Entity(*name),
            Self::Relation { name, .. } => PersistentNameKey::Relation(*name),
            Self::Group { group } => PersistentNameKey::Group(*group),
        }
    }
}

/// How a held name chooses among its live descendants at the basis.
///
/// Every selection first collects the live descendants (the held name itself
/// when it is live). With none live, every selection resolves `Retired`.
/// `Exact` refuses `Ambiguous` on two or more; `AllDescendants` returns them
/// all as `Plural`. `BoundedByCarriers` keeps descendants whose carrier set
/// contains every requested carrier; `KeepSide` keeps descendants born with
/// exactly this canonical discriminator (groups carry none). A filter
/// resolves one match `Exact`, refuses several `Ambiguous`, and refuses zero
/// `NoMatchingCandidate`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersistentNameSelection {
    Exact,
    AllDescendants,
    BoundedByCarriers {
        carriers: BTreeSet<PersistentNameIdentity>,
    },
    KeepSide {
        discriminator: Vec<u8>,
    },
}

/// Successful resolution of a held name at a basis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersistentNameResolution {
    Exact(PersistentNameTarget),
    /// Two or more live descendants in ascending [`PersistentNameKey`] order.
    Plural(Vec<PersistentNameTarget>),
    /// No live descendant; the latest commit at which a reached name ended.
    Retired {
        at_commit: CommitId,
    },
}

/// Typed refusal to resolve a held name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersistentNameResolutionDenial {
    /// Two or more candidates, ascending, that the selection cannot narrow.
    Ambiguous { candidates: Vec<PersistentNameKey> },
    /// A carrier or side filter matched none of these live descendants.
    NoMatchingCandidate {
        live_candidates: Vec<PersistentNameKey>,
    },
    /// The basis lies below the index's history retention floor.
    BasisPruned {
        basis: CommitId,
        retention_floor: CommitId,
    },
    /// The held name has no birth at or before the basis.
    UnknownName { name: PersistentNameKey },
}

/// Exact work one resolution performed, independent of unrelated history.
///
/// `index_probes` counts name-entry lookups, `event_visits` the ending
/// transitions read for names that were no longer live, and `edge_visits`
/// the forward lineage edges followed from those transitions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PersistentNameResolutionMetrics {
    pub index_probes: usize,
    pub edge_visits: usize,
    pub event_visits: usize,
}

/// One resolution answer with the work it cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistentNameResolutionOutcome {
    pub result: Result<PersistentNameResolution, PersistentNameResolutionDenial>,
    pub metrics: PersistentNameResolutionMetrics,
}
