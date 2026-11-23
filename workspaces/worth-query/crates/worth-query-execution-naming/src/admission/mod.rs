//! Sole production runtime admission boundary for elevating Relational-minted
//! persistent name digests into kind-typed [`WorthQueryName`].
//!
//! Boundary enforcement (`tools/boundary-check/config/road1.toml`) restricts
//! `worth_query_declaration::persistent_name_admission_authority` to this
//! module and its declaration owner so no external crate, handler, or caller
//! can forge a `WorthQueryName<K>` from raw bytes.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::aggregate_identity::AggregateGroupAssignment;
use worth_relational::facade::identity::{EntityId, LineageId, RelationId};
use worth_relational::facade::lineage::{
    LineageEventKind, LineageSubject, PersistentNameBirthRecord, PersistentNameIdentity,
    PersistentNameKey, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{CommitResult, CreatedEntityRef, CreatedRelationRef};

use crate::denial::ExecutionNamingDenial;

/// Elevates one Relational-minted [`PersistentNameBirthRecord`] into a
/// kind-typed [`WorthQueryName`] after verifying its subject class matches
/// `expected_kind`.
pub fn admit_minted_birth_record<K>(
    record: &PersistentNameBirthRecord,
    expected_kind: PersistentNameSubjectKind,
) -> Result<WorthQueryName<K>, ExecutionNamingDenial> {
    if record.subject_kind() != expected_kind {
        return Err(ExecutionNamingDenial::AdmissionSubjectKindMismatch {
            expected: expected_kind,
            actual: record.subject_kind(),
        });
    }
    Ok(admit_minted_digest(*record.identity().as_bytes()))
}

/// Elevates a Relational-minted [`AggregateGroupAssignment`] into a kind-typed
/// group [`WorthQueryName`].
pub fn admit_group_assignment<G>(assignment: &AggregateGroupAssignment) -> WorthQueryName<G> {
    admit_minted_digest(*assignment.identity.bytes())
}

/// Elevates a Relational-resolved [`PersistentNameKey`] into a kind-typed
/// [`WorthQueryName`] after verifying its subject class.
pub fn admit_resolved_name_key<K>(
    key: PersistentNameKey,
    expected_kind: PersistentNameSubjectKind,
) -> Result<WorthQueryName<K>, ExecutionNamingDenial> {
    if key.subject_kind() != expected_kind {
        return Err(ExecutionNamingDenial::AdmissionSubjectKindMismatch {
            expected: expected_kind,
            actual: key.subject_kind(),
        });
    }
    Ok(admit_minted_digest(*key.digest()))
}

/// Sealed collection of persistent names minted during one committed
/// Relational transaction, extracted exclusively from the commit's canonical
/// `LineageEventRecord`s and Relational's authoritative lineage bindings.
#[derive(Clone, Debug)]
pub struct AdmittedCommitNames {
    entity_births: BTreeMap<EntityId, PersistentNameBirthRecord>,
    relation_births: BTreeMap<RelationId, PersistentNameBirthRecord>,
    by_lineage: BTreeMap<LineageId, PersistentNameBirthRecord>,
}

impl AdmittedCommitNames {
    pub(crate) fn from_committed_result(
        runtime: &RelationalRuntime,
        commit_result: &CommitResult,
    ) -> Self {
        let mut by_lineage = BTreeMap::new();
        for event in commit_result.publication().envelope.lineage_events() {
            if event.kind() == LineageEventKind::Create
                && let Some(birth) = event.name_birth()
                && let Some(&lineage_id) = event.targets().first()
            {
                by_lineage.insert(lineage_id, birth.clone());
            }
        }

        let mut entity_births = BTreeMap::new();
        let mut relation_births = BTreeMap::new();
        let live_names = runtime.branch_live_persistent_names(&commit_result.commit.branch_id);
        for birth in by_lineage.values() {
            let Some(holder) = live_names.get(&birth.identity()) else {
                continue;
            };
            match (birth.subject_kind(), holder) {
                (
                    PersistentNameSubjectKind::Entity,
                    worth_relational::facade::lineage::PersistentNameHolder::Entity(entity_id),
                ) => {
                    entity_births.insert(*entity_id, birth.clone());
                }
                (
                    PersistentNameSubjectKind::Relation,
                    worth_relational::facade::lineage::PersistentNameHolder::Relation(relation_id),
                ) => {
                    relation_births.insert(*relation_id, birth.clone());
                }
                _ => {}
            }
        }

        // Also bind relations directly via Relational's persisted relation
        // lineage map for any relation created and retired/replaced in-commit.
        for event in commit_result.publication().envelope.lineage_events() {
            if event.kind() == LineageEventKind::Create
                && event.subject() == LineageSubject::Relation
                && let Some(birth) = event.name_birth()
                && let Some(&lineage_id) = event.targets().first()
            {
                by_lineage.entry(lineage_id).or_insert_with(|| birth.clone());
            }
        }

        Self {
            entity_births,
            relation_births,
            by_lineage,
        }
    }

    /// Returns the admitted entity [`WorthQueryName`] minted for `entity_id` in
    /// this commit, or refuses typed if `entity_id` had no committed birth.
    pub fn entity_name<K>(
        &self,
        entity_id: EntityId,
    ) -> Result<WorthQueryName<K>, ExecutionNamingDenial> {
        let record = self.entity_birth(entity_id)?;
        admit_minted_birth_record(record, PersistentNameSubjectKind::Entity)
    }

    /// Returns the admitted entity [`WorthQueryName`] for a `CreatedEntityRef`
    /// resolved through the same sealed `CommitResult`.
    pub fn created_entity_name<K>(
        &self,
        commit_result: &CommitResult,
        created: &CreatedEntityRef,
    ) -> Result<(EntityId, WorthQueryName<K>), ExecutionNamingDenial> {
        let entity_id = commit_result.created_entity(created).ok_or_else(|| {
            ExecutionNamingDenial::UnadmittedBirth {
                detail: format!("created entity reference {:?} was not allocated in commit", created.client_key),
            }
        })?;
        let name = self.entity_name(entity_id)?;
        Ok((entity_id, name))
    }

    /// Returns the admitted relation [`WorthQueryName`] minted for `relation_id`
    /// in this commit, or refuses typed if `relation_id` had no committed birth.
    pub fn relation_name<K>(
        &self,
        relation_id: RelationId,
    ) -> Result<WorthQueryName<K>, ExecutionNamingDenial> {
        let record = self.relation_birth(relation_id)?;
        admit_minted_birth_record(record, PersistentNameSubjectKind::Relation)
    }

    /// Returns the admitted relation [`WorthQueryName`] for a
    /// `CreatedRelationRef` resolved through the same sealed `CommitResult`.
    pub fn created_relation_name<K>(
        &self,
        commit_result: &CommitResult,
        created: &CreatedRelationRef,
    ) -> Result<(RelationId, WorthQueryName<K>), ExecutionNamingDenial> {
        let relation_id = commit_result.created_relation(created).ok_or_else(|| {
            ExecutionNamingDenial::UnadmittedBirth {
                detail: format!("created relation reference {:?} was not allocated in commit", created.client_key),
            }
        })?;
        let name = self.relation_name(relation_id)?;
        Ok((relation_id, name))
    }

    pub fn entity_birth(
        &self,
        entity_id: EntityId,
    ) -> Result<&PersistentNameBirthRecord, ExecutionNamingDenial> {
        self.entity_births
            .get(&entity_id)
            .ok_or_else(|| ExecutionNamingDenial::UnadmittedBirth {
                detail: format!("entity {entity_id:?} has no committed persistent name birth"),
            })
    }

    pub fn relation_birth(
        &self,
        relation_id: RelationId,
    ) -> Result<&PersistentNameBirthRecord, ExecutionNamingDenial> {
        self.relation_births
            .get(&relation_id)
            .ok_or_else(|| ExecutionNamingDenial::UnadmittedBirth {
                detail: format!("relation {relation_id:?} has no committed persistent name birth"),
            })
    }

    pub fn birth_count(&self) -> usize {
        self.by_lineage.len()
    }

    pub fn all_births(&self) -> impl ExactSizeIterator<Item = &PersistentNameBirthRecord> {
        self.by_lineage.values()
    }

    /// Returns the Relational `PersistentNameIdentity` for a committed entity
    /// birth (for passing into downstream Relational lineage postures).
    pub fn entity_relational_identity(
        &self,
        entity_id: EntityId,
    ) -> Result<PersistentNameIdentity, ExecutionNamingDenial> {
        Ok(self.entity_birth(entity_id)?.identity())
    }

    /// Returns the Relational `PersistentNameIdentity` for a committed relation
    /// birth.
    pub fn relation_relational_identity(
        &self,
        relation_id: RelationId,
    ) -> Result<PersistentNameIdentity, ExecutionNamingDenial> {
        Ok(self.relation_birth(relation_id)?.identity())
    }
}
