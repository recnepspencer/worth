use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::transactions::data::RecordRef;

use super::events::LineageSubject;
use super::naming::{
    InstalledPersistentNameSchemeRegistry, PersistentNameBirthRequest, PersistentNameHolder,
    PersistentNameIdentity, PersistentNameSubjectKind,
};

/// Exact entity or relation record targeted by a Phase 2 lineage posture claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LineageRecordTarget {
    Entity(EntityId),
    Relation(RelationId),
}

impl LineageRecordTarget {
    pub const fn subject(self) -> LineageSubject {
        match self {
            Self::Entity(_) => LineageSubject::Entity,
            Self::Relation(_) => LineageSubject::Relation,
        }
    }

    pub const fn subject_kind(self) -> PersistentNameSubjectKind {
        match self {
            Self::Entity(_) => PersistentNameSubjectKind::Entity,
            Self::Relation(_) => PersistentNameSubjectKind::Relation,
        }
    }

    pub const fn to_holder(self) -> PersistentNameHolder {
        match self {
            Self::Entity(entity_id) => PersistentNameHolder::Entity(entity_id),
            Self::Relation(relation_id) => PersistentNameHolder::Relation(relation_id),
        }
    }

    pub const fn to_record_ref(self) -> RecordRef {
        match self {
            Self::Entity(entity_id) => RecordRef::Entity(entity_id),
            Self::Relation(relation_id) => RecordRef::Relation(relation_id),
        }
    }
}

/// One newly created successor piece of a [`LineagePostureClaim::Split`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageSplitChild {
    pub target: LineageRecordTarget,
    pub birth: PersistentNameBirthRequest,
}

/// Keeper rule for a [`LineagePostureClaim::Merge`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageMergeKeeper {
    /// Preserves one listed parent's persistent name; must be a member of the
    /// merge's `parents` list.
    KeepParent(PersistentNameIdentity),
    /// Mints a distinct persistent name carrying all sorted parent identities
    /// and a canonical discriminator.
    MintNew { birth: PersistentNameBirthRequest },
}

/// Explicit entity or relation output posture claim validated against actual
/// staged commit effects before lineage IDs or events are emitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineagePostureClaim {
    Create {
        target: LineageRecordTarget,
        birth: PersistentNameBirthRequest,
    },
    Preserve {
        target: LineageRecordTarget,
        name: PersistentNameIdentity,
    },
    Retire {
        target: LineageRecordTarget,
        name: PersistentNameIdentity,
    },
    Split {
        parent_target: LineageRecordTarget,
        parent_name: PersistentNameIdentity,
        children: Vec<LineageSplitChild>,
    },
    Merge {
        parents: Vec<(LineageRecordTarget, PersistentNameIdentity)>,
        target: LineageRecordTarget,
        keeper: LineageMergeKeeper,
    },
}

/// Basis state and declared posture claims supplied to Relational commit
/// finalization for many-to-many entity and relation lineage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineageCommitPostureInput {
    pub scheme_registry: InstalledPersistentNameSchemeRegistry,
    pub live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    pub postures: Vec<LineagePostureClaim>,
}
