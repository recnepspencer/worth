#[cfg(test)]
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::identity::data::{EntityId, LineageId, RelationId};
use crate::transactions::data::primitives::RecordRef;

use super::events::LineageSubject;
#[cfg(test)]
use super::naming::InstalledPersistentNameSchemeRegistry;
use super::naming::{
    PersistentNameBirthRequest, PersistentNameDiscriminatorReference, PersistentNameHolder,
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
    /// Explicit cross-commit restore re-activating a previously retired
    /// persistent name onto a newly created target record (undo, redo,
    /// history checkout, or authored restore).
    Restore {
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

/// Unresolved creation target inside a [`CommitNamingPlan`], bound by
/// Relational to the exact `EntityId` or `RelationId` allocated during commit
/// execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingBirthTarget {
    CreatedEntity(crate::transactions::data::primitives::CreatedEntityRef),
    CreatedRelation(crate::transactions::data::primitives::CreatedRelationRef),
}

/// One pending persistent-name birth attached to a transaction before
/// Relational allocates the created entity or relation record identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCommitBirth {
    pub target: PendingBirthTarget,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub parent_identities: Vec<PersistentNameIdentity>,
    pub referenced_names: Vec<PersistentNameDiscriminatorReference>,
    pub canonical_discriminator: Vec<u8>,
}

/// One unresolved successor child inside a [`PendingSplitClaim`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSplitChild {
    pub target: PendingBirthTarget,
    pub scheme_id: String,
    pub occurrence_key: String,
    pub role_path: String,
    pub referenced_names: Vec<PersistentNameDiscriminatorReference>,
    pub canonical_discriminator: Vec<u8>,
}

/// Unresolved `Split` posture attached to a [`CommitNamingPlan`] whose child
/// targets are resolved by Relational against created record allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSplitClaim {
    pub parent_target: LineageRecordTarget,
    pub parent_name: PersistentNameIdentity,
    pub children: Vec<PendingSplitChild>,
}

/// Target of a [`PendingMergeClaim`]: either an existing in-place keeper or a
/// newly created record in this commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingMergeTarget {
    Existing(LineageRecordTarget),
    Created(PendingBirthTarget),
}

/// Keeper rule for a [`PendingMergeClaim`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingMergeKeeper {
    KeepParent(PersistentNameIdentity),
    MintNew {
        scheme_id: String,
        occurrence_key: String,
        role_path: String,
        referenced_names: Vec<PersistentNameDiscriminatorReference>,
        canonical_discriminator: Vec<u8>,
    },
}

/// Unresolved `Merge` posture attached to a [`CommitNamingPlan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMergeClaim {
    pub parents: Vec<(LineageRecordTarget, PersistentNameIdentity)>,
    pub target: PendingMergeTarget,
    pub keeper: PendingMergeKeeper,
}

/// Commit-time persistent naming and lineage posture plan attached to a
/// [`crate::mvcc::BranchBoundRelationalTransaction`].
///
/// Intentionally carries NO `scheme_registry`, `live_holders`, `live_lineages`,
/// or `retired_lineages`: Relational reads installed schemes exclusively from
/// its own schema authority and derives live/retired lineage sets exclusively
/// from its own authoritative `LineageSubsystem` at commit finalization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitNamingPlan {
    pub pending_births: Vec<PendingCommitBirth>,
    pub pending_splits: Vec<PendingSplitClaim>,
    pub pending_merges: Vec<PendingMergeClaim>,
    pub postures: Vec<LineagePostureClaim>,
}

/// Borrowed point-lookup view over authoritative branch naming state so commit
/// finalization performs O(delta) lookups without cloning branch maps.
pub(crate) trait LineageBasisView {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder>;
    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId>;
    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)>;
    /// The persistent name `holder` last held on this branch, live or retired.
    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity>;

    fn contains_live_holder(&self, id: &PersistentNameIdentity) -> bool {
        self.live_holder(id).is_some()
    }

    fn contains_retired(&self, id: &PersistentNameIdentity) -> bool {
        self.retired_lineage(id).is_some()
    }
}

/// Internal commit finalization posture input (test-only helper).
#[cfg(test)]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineageCommitPostureInput {
    pub scheme_registry: InstalledPersistentNameSchemeRegistry,
    pub live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    /// Previously retired persistent names and their most recent retired
    /// `(LineageSubject, LineageId)` at the commit's basis.
    pub retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
    pub postures: Vec<LineagePostureClaim>,
}

#[cfg(test)]
impl LineageBasisView for LineageCommitPostureInput {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
        self.live_holders.get(id)
    }

    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.live_lineages.get(holder).copied()
    }

    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.retired_lineages.get(id).copied()
    }

    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        self.live_holders
            .iter()
            .find_map(|(name, live)| (live == holder).then_some(*name))
    }
}
