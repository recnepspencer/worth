//! Sole production runtime admission boundary for elevating Relational-sealed
//! persistent-name births into kind-typed [`WorthQueryName`].
//!
//! No public function here accepts raw bytes, caller-constructed birth structs,
//! or unconstrained kind parameters `K`. Every [`WorthQueryName`] is minted
//! exclusively from a Relational-sealed [`CommitResult`] inside
//! [`AdmittedCommitNames`] after verifying BOTH subject class and the declared
//! entity/relation marker name against Relational's own installed scheme
//! registry.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    PersistentNameBirthRecord, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{CommitResult, CreatedEntityRef, CreatedRelationRef};

use crate::denial::ExecutionNamingDenial;

mod resolved;
mod same_commit;
pub(crate) use resolved::{kind_verified_resolved_name, verify_resolved_subject, RequestedSubject};
pub use same_commit::same_commit_name_of;

/// Sealed collection of persistent names minted during one committed
/// Relational transaction, derived strictly from that commit's own canonical
/// lineage artifact and Relational's installed scheme subject names.
///
/// Construction is crate-private (`from_committed_result`), and lookups run in
/// O(log commit births) with zero runtime lock acquisitions and immunity to
/// concurrent post-commit retirements.
#[derive(Clone, Debug)]
pub struct AdmittedCommitNames {
    entity_births: BTreeMap<EntityId, PersistentNameBirthRecord>,
    relation_births: BTreeMap<RelationId, PersistentNameBirthRecord>,
    scheme_subjects: BTreeMap<String, (PersistentNameSubjectKind, String)>,
}

impl AdmittedCommitNames {
    pub(crate) fn from_committed_result(
        runtime: &RelationalRuntime,
        commit_result: &CommitResult,
    ) -> Self {
        let mut entity_births = BTreeMap::new();
        let mut relation_births = BTreeMap::new();
        let mut scheme_subjects = BTreeMap::new();

        for (holder, birth) in commit_result.committed_name_births() {
            let scheme_id = birth.parts().scheme_id();
            if !scheme_subjects.contains_key(scheme_id)
                && let Some(subject) = runtime.persistent_name_scheme_subject(scheme_id)
            {
                scheme_subjects.insert(scheme_id.to_owned(), subject);
            }
            match (birth.subject_kind(), holder) {
                (PersistentNameSubjectKind::Entity, PersistentNameHolder::Entity(entity_id)) => {
                    entity_births.insert(entity_id, birth);
                }
                (
                    PersistentNameSubjectKind::Relation,
                    PersistentNameHolder::Relation(relation_id),
                ) => {
                    relation_births.insert(relation_id, birth);
                }
                _ => {}
            }
        }

        Self {
            entity_births,
            relation_births,
            scheme_subjects,
        }
    }

    /// Returns the admitted entity [`WorthQueryName<Entity>`] minted for
    /// `entity_id` in this commit after verifying `Entity`'s declared schema
    /// identifier matches the birth scheme's installed subject name.
    pub fn entity_name<Schema, Entity>(
        &self,
        entity_id: EntityId,
    ) -> Result<WorthQueryName<Entity>, ExecutionNamingDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let record = self.entity_birth(entity_id)?;
        self.admit_verified_record(
            record,
            PersistentNameSubjectKind::Entity,
            Entity::IDENTIFIER,
        )
    }

    /// Returns the admitted entity [`WorthQueryName<Entity>`] for a
    /// `CreatedEntityRef` resolved through the same sealed `CommitResult`.
    pub fn created_entity_name<Schema, Entity>(
        &self,
        commit_result: &CommitResult,
        created: &CreatedEntityRef,
    ) -> Result<(EntityId, WorthQueryName<Entity>), ExecutionNamingDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let entity_id = commit_result.created_entity(created).ok_or_else(|| {
            ExecutionNamingDenial::UnadmittedBirth {
                detail: format!(
                    "created entity reference {:?} was not allocated in commit",
                    created.client_key
                ),
            }
        })?;
        let name = self.entity_name::<Schema, Entity>(entity_id)?;
        Ok((entity_id, name))
    }

    /// Returns the admitted relation [`WorthQueryName<Relation>`] minted for
    /// `relation_id` in this commit after verifying `Relation::NAME` matches
    /// the birth scheme's installed subject name.
    pub fn relation_name<Relation>(
        &self,
        relation_id: RelationId,
    ) -> Result<WorthQueryName<Relation>, ExecutionNamingDenial>
    where
        Relation: fragment::Relation,
    {
        let record = self.relation_birth(relation_id)?;
        self.admit_verified_record(record, PersistentNameSubjectKind::Relation, Relation::NAME)
    }

    /// Returns the admitted relation [`WorthQueryName<Relation>`] for a
    /// `CreatedRelationRef` resolved through the same sealed `CommitResult`.
    pub fn created_relation_name<Relation>(
        &self,
        commit_result: &CommitResult,
        created: &CreatedRelationRef,
    ) -> Result<(RelationId, WorthQueryName<Relation>), ExecutionNamingDenial>
    where
        Relation: fragment::Relation,
    {
        let relation_id = commit_result.created_relation(created).ok_or_else(|| {
            ExecutionNamingDenial::UnadmittedBirth {
                detail: format!(
                    "created relation reference {:?} was not allocated in commit",
                    created.client_key
                ),
            }
        })?;
        let name = self.relation_name::<Relation>(relation_id)?;
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
        self.entity_births.len() + self.relation_births.len()
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

    fn admit_verified_record<K>(
        &self,
        record: &PersistentNameBirthRecord,
        expected_kind: PersistentNameSubjectKind,
        requested_subject: &str,
    ) -> Result<WorthQueryName<K>, ExecutionNamingDenial> {
        if record.subject_kind() != expected_kind {
            return Err(ExecutionNamingDenial::AdmissionSubjectKindMismatch {
                expected: expected_kind,
                actual: record.subject_kind(),
            });
        }
        let scheme_id = record.parts().scheme_id();
        let Some((installed_kind, installed_subject)) = self.scheme_subjects.get(scheme_id) else {
            return Err(ExecutionNamingDenial::UnknownScheme {
                scheme_id: scheme_id.to_owned(),
            });
        };
        if *installed_kind != expected_kind {
            return Err(ExecutionNamingDenial::AdmissionSubjectKindMismatch {
                expected: expected_kind,
                actual: *installed_kind,
            });
        }
        if installed_subject != requested_subject {
            return Err(ExecutionNamingDenial::KindIdentityMismatch {
                scheme_id: scheme_id.to_owned(),
                expected_subject: installed_subject.clone(),
                requested_subject: requested_subject.to_owned(),
            });
        }
        Ok(admit_minted_digest(*record.identity().as_bytes()))
    }
}
