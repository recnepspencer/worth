use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::{
    LineageRecordTarget, LineageSubject, PersistentNameBirthRecord, PersistentNameIdentity,
};

pub(in super::super) enum ValidatedMergeTarget {
    InPlaceKeeper {
        lineage_id: LineageId,
        kept_name: PersistentNameIdentity,
    },
    RecreatedKeeper {
        target: LineageRecordTarget,
        kept_name: PersistentNameIdentity,
    },
    MintedNew {
        target: LineageRecordTarget,
    },
}

pub(in super::super) enum ValidatedPosture {
    Create {
        target: LineageRecordTarget,
    },
    Preserve,
    Regenerate {
        subject: LineageSubject,
        source_lineage: LineageId,
        target: LineageRecordTarget,
        name: PersistentNameIdentity,
    },
    Restore {
        target: LineageRecordTarget,
        source_lineage: LineageId,
        name: PersistentNameIdentity,
    },
    Retire {
        subject: LineageSubject,
        source_lineage: LineageId,
        name: PersistentNameIdentity,
    },
    Split {
        subject: LineageSubject,
        parent_lineage: LineageId,
        children: Vec<LineageRecordTarget>,
    },
    Merge {
        subject: LineageSubject,
        source_lineages: Vec<LineageId>,
        target: ValidatedMergeTarget,
    },
}

pub(in super::super) struct ValidatedPostureBatch {
    pub(in super::super) postures: Vec<ValidatedPosture>,
    pub(in super::super) minted_births: BTreeMap<LineageRecordTarget, PersistentNameBirthRecord>,
    pub(in super::super) claimed_created_targets: BTreeSet<LineageRecordTarget>,
    pub(in super::super) restored_created_targets: BTreeSet<LineageRecordTarget>,
    pub(in super::super) claimed_basis_targets: BTreeSet<LineageRecordTarget>,
}
