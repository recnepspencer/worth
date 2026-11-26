//! The persistent names the commit under evaluation gives its records, read
//! before the commit seals.
//!
//! A preview runs the commit's own naming authority: the naming plan lowers
//! through [`resolve_commit_naming_plan`] and validates and mints through
//! [`validate_commit_postures`], exactly as finalization does. So a pending
//! birth answers with the identity and canonical parts finalization records,
//! and a plan finalization refuses is refused here with the same typed
//! [`LineageNameDenial`]. Nothing is reserved, allocated or written.

use std::collections::BTreeMap;

use crate::contracts::identity::data::primitives::VersionId;
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::contracts::lineage::data::{
    CommitNamingPlan, InstalledPersistentNameSchemeRegistry, LineageBasisView, LineageMergeKeeper,
    LineageNameDenial, LineagePostureClaim, LineageRecordTarget, PersistentNameBirthRecord,
    PersistentNameIdentity, PersistentNameSubjectKind,
};
use crate::contracts::schema::data::RelationalSchemaRegistry;
use crate::contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use crate::contracts::transactions::data::primitives::RecordRef;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use super::commit_finalization::resolve_commit_naming_plan;
use super::posture_validation::{validate_commit_postures, ValidatedPosture};

/// The proposed state of one commit, as its naming authority sees it.
pub struct CommitNamePreviewInput<'a> {
    pub staged: &'a WorkingState,
    pub version_id: VersionId,
    pub changed_records: &'a [RecordRef],
    pub created_entities: &'a CommitCreatedEntityBindings,
    pub created_relations: &'a CommitCreatedRelationBindings,
    pub naming_plan: Option<&'a CommitNamingPlan>,
    pub scheme_registry: &'a InstalledPersistentNameSchemeRegistry,
    pub schema_registry: Option<&'a RelationalSchemaRegistry>,
    pub basis: &'a dyn LineageBasisView,
}

/// What the commit does to one record's name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingRecordName {
    /// The record carries this name once the commit seals: a birth, a kept
    /// merge or restored name.
    Named(PersistentNameKey),
    /// The commit ends the record's name: a retire, split or merged-away
    /// parent.
    Ended,
}

/// The name changes one validated commit makes, over its committed basis.
///
/// Bounded by the commit's own naming plan: it never copies the branch's
/// committed names.
#[derive(Debug, Default)]
pub struct PendingCommitNames {
    records: BTreeMap<LineageRecordTarget, PendingRecordName>,
    births: BTreeMap<[u8; PersistentNameIdentity::BYTE_LEN], PersistentNameBirthRecord>,
}

impl PendingCommitNames {
    /// What the commit does to `record`'s name; `None` leaves it as committed.
    ///
    /// Complexity: O(log r) in the commit's r named records.
    pub fn record_name(&self, record: LineageRecordTarget) -> Option<PendingRecordName> {
        self.records.get(&record).copied()
    }

    /// The key of a name this commit mints, when `digest` is one under
    /// `subject`.
    ///
    /// Complexity: O(log b) in the commit's b births.
    pub fn recorded_name(
        &self,
        subject: PersistentNameSubjectKind,
        digest: &[u8; PersistentNameIdentity::BYTE_LEN],
    ) -> Option<PersistentNameKey> {
        let birth = self.births.get(digest)?;
        if birth.subject_kind() != subject {
            return None;
        }
        name_key(subject, birth.identity())
    }

    /// The birth this commit mints for `name`.
    ///
    /// Complexity: O(log b) in the commit's b births.
    pub fn birth_of(&self, name: PersistentNameKey) -> Option<&PersistentNameBirthRecord> {
        let birth = self.births.get(name.digest())?;
        (birth.subject_kind() == name.subject_kind()).then_some(birth)
    }

    fn name(&mut self, target: LineageRecordTarget, name: PersistentNameIdentity) {
        if let Some(key) = name_key(target.subject_kind(), name) {
            self.records.insert(target, PendingRecordName::Named(key));
        }
    }

    fn end(&mut self, target: LineageRecordTarget) {
        self.records
            .entry(target)
            .or_insert(PendingRecordName::Ended);
    }
}

/// Validates and mints the commit's names exactly as finalization does,
/// without reserving lineage or writing anything.
///
/// Complexity: as `prepare_posture_finalization`'s validation: O(changed
/// records + postures) plus minting each birth.
pub fn preview_commit_names(
    input: CommitNamePreviewInput<'_>,
) -> Result<PendingCommitNames, LineageNameDenial> {
    let postures = match input.naming_plan {
        Some(plan) => {
            resolve_commit_naming_plan(plan, input.created_entities, input.created_relations)?
        }
        None => Vec::new(),
    };
    let validated = validate_commit_postures(
        input.staged,
        input.version_id,
        input.changed_records,
        input.scheme_registry,
        input.schema_registry,
        input.basis,
        &postures,
    )?;
    let mut names = PendingCommitNames::default();
    for claim in &postures {
        match claim {
            LineagePostureClaim::Retire { target, .. } => names.end(*target),
            LineagePostureClaim::Split { parent_target, .. } => names.end(*parent_target),
            LineagePostureClaim::Restore { target, name } => names.name(*target, *name),
            LineagePostureClaim::Merge {
                parents,
                target,
                keeper,
            } => {
                if let LineageMergeKeeper::KeepParent(kept) = keeper {
                    names.name(*target, *kept);
                }
                parents
                    .iter()
                    .filter(|(parent, _)| parent != target)
                    .for_each(|(parent, _)| names.end(*parent));
            }
            LineagePostureClaim::Create { .. } | LineagePostureClaim::Preserve { .. } => {}
        }
    }
    for posture in &validated.postures {
        if let ValidatedPosture::Regenerate { target, name, .. } = posture {
            names.name(*target, *name);
        }
    }
    for (target, birth) in validated.minted_births {
        names.name(target, birth.identity());
        names.births.insert(*birth.identity().as_bytes(), birth);
    }
    Ok(names)
}

fn name_key(
    subject: PersistentNameSubjectKind,
    name: PersistentNameIdentity,
) -> Option<PersistentNameKey> {
    match subject {
        PersistentNameSubjectKind::Entity => Some(PersistentNameKey::Entity(name)),
        PersistentNameSubjectKind::Relation => Some(PersistentNameKey::Relation(name)),
        PersistentNameSubjectKind::Group => None,
    }
}
