//! Tracked reads of recorded births: a held name's birth, and the name a
//! birth of given parts was minted as.
//!
//! A birth is immutable once recorded, and stays visible at every later
//! basis of its branch: undo, checkout and history navigation retire and
//! restore names, never un-record a birth. So a read that found a birth
//! rests on nothing a later commit changes, and records nothing. A read
//! that found no birth records the awaited digest, which the commit that
//! records it stales (`NameReadStaleness::births`).

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameSubjectKind,
};

use super::{ExecutionNameReadDenial, ExecutionNameReader, NameReadDependencies};
use crate::admission::{
    admit_birth_of, kind_verified_resolved_name, verify_resolved_subject, RequestedSubject,
};
use crate::operation_outputs::WorthQueryOutputBirth;

type Read<T> = Result<T, ExecutionNameReadDenial>;

const fn entity<Schema, Entity: ApplicationEntityMarkerIdentity<Schema>>(
) -> RequestedSubject<'static> {
    RequestedSubject {
        kind: PersistentNameSubjectKind::Entity,
        name: Entity::IDENTIFIER,
    }
}

const fn relation<Relation: fragment::Relation>() -> RequestedSubject<'static> {
    RequestedSubject {
        kind: PersistentNameSubjectKind::Relation,
        name: Relation::NAME,
    }
}

impl ExecutionNameReader<'_> {
    /// The recorded birth of a held entity name: its occurrence, role,
    /// parents and discriminator, decoded with
    /// [`RecordedNameBirth::decode`].
    ///
    /// Answers for any name born at or before the basis, live or not: a
    /// split, merged or retired name still has its birth. Refuses
    /// `UnknownName` for a name not recorded at the basis, recording the
    /// awaited birth.
    ///
    /// Complexity: O(1) expected index probes plus O(parents + B) for the B
    /// discriminator bytes.
    pub fn entity_birth_of<Schema, Entity>(
        &self,
        held: &WorthQueryName<Entity>,
        dependencies: &mut NameReadDependencies,
    ) -> Read<RecordedNameBirth<Entity>>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        self.birth_of(held, entity::<Schema, Entity>(), dependencies)
    }

    /// The recorded birth of a held relation name.
    ///
    /// Complexity: as [`Self::entity_birth_of`].
    pub fn relation_birth_of<Relation>(
        &self,
        held: &WorthQueryName<Relation>,
        dependencies: &mut NameReadDependencies,
    ) -> Read<RecordedNameBirth<Relation>>
    where
        Relation: fragment::Relation,
    {
        self.birth_of(held, relation::<Relation>(), dependencies)
    }

    /// The entity name a birth of exactly these parts was minted as: the
    /// installed scheme's identity of `birth` under `parents`, admitted only
    /// when that birth is recorded and visible at the basis.
    ///
    /// `Ok(None)` when no such birth is recorded yet (or the birth is on
    /// another branch); the read then records the awaited birth, so the
    /// commit that records it stales the answer. The answer is a name, not a
    /// record: resolve it to find where its meaning lives now. Refuses a
    /// `birth` whose discriminator references a same-commit name.
    ///
    /// Complexity: O(B + P log P) for the B birth bytes and P parents, plus
    /// O(1) expected index probes.
    pub fn entity_name_for_birth<Schema, Entity>(
        &self,
        birth: &WorthQueryOutputBirth,
        parents: &[WorthQueryName<Entity>],
        dependencies: &mut NameReadDependencies,
    ) -> Read<Option<WorthQueryName<Entity>>>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        self.name_for_birth(birth, parents, entity::<Schema, Entity>(), dependencies)
    }

    /// The relation name a birth of exactly these parts was minted as.
    ///
    /// Complexity: as [`Self::entity_name_for_birth`].
    pub fn relation_name_for_birth<Relation>(
        &self,
        birth: &WorthQueryOutputBirth,
        parents: &[WorthQueryName<Relation>],
        dependencies: &mut NameReadDependencies,
    ) -> Read<Option<WorthQueryName<Relation>>>
    where
        Relation: fragment::Relation,
    {
        self.name_for_birth(birth, parents, relation::<Relation>(), dependencies)
    }

    fn birth_of<K>(
        &self,
        held: &WorthQueryName<K>,
        requested: RequestedSubject<'_>,
        dependencies: &mut NameReadDependencies,
    ) -> Read<RecordedNameBirth<K>> {
        let branch = self.branch().clone();
        self.with_view(|view| {
            let Some(key) = view.recorded_name(requested.kind, held.as_bytes()) else {
                dependencies.record_awaited_birth(&branch, *held.as_bytes());
                return Err(ExecutionNameReadDenial::UnknownName);
            };
            admit_birth_of(self.runtime, &view, key, requested)
        })?
    }

    fn name_for_birth<K>(
        &self,
        birth: &WorthQueryOutputBirth,
        parents: &[WorthQueryName<K>],
        requested: RequestedSubject<'_>,
        dependencies: &mut NameReadDependencies,
    ) -> Read<Option<WorthQueryName<K>>> {
        let discriminator = birth
            .committed_discriminator()
            .ok_or(ExecutionNameReadDenial::SameCommitBirthReference)?;
        let scheme_id = self
            .runtime
            .persistent_name_scheme_id(requested.kind, requested.name)
            .ok_or(ExecutionNameReadDenial::UnknownScheme { scheme_id: None })?;
        let branch = self.branch().clone();
        self.with_view(|view| {
            // A parent not yet recorded has no identity here, and no birth
            // under it can be recorded before it is.
            let mut identities = Vec::with_capacity(parents.len());
            for parent in parents {
                let Some(key) = view.recorded_name(requested.kind, parent.as_bytes()) else {
                    dependencies.record_awaited_birth(&branch, *parent.as_bytes());
                    return Ok(None);
                };
                // A recorded parent of another kind refuses, as every read
                // of a held name does, rather than awaiting a birth that can
                // never be recorded.
                verify_resolved_subject(self.runtime, &view, key, requested)?;
                match key {
                    PersistentNameKey::Entity(id) | PersistentNameKey::Relation(id) => {
                        identities.push(id);
                    }
                    PersistentNameKey::Group(_) => {
                        return Err(ExecutionNameReadDenial::SubjectKindMismatch {
                            expected: requested.kind,
                            actual: PersistentNameSubjectKind::Group,
                        });
                    }
                }
            }
            let minted = PersistentNameIdentity::of_pending_birth(
                &scheme_id,
                birth.occurrence_key(),
                birth.role_path(),
                &identities,
                discriminator,
            );
            match view.recorded_name(requested.kind, minted.as_bytes()) {
                Some(key) => {
                    kind_verified_resolved_name(self.runtime, &view, key, requested).map(Some)
                }
                None => {
                    dependencies.record_awaited_birth(&branch, *minted.as_bytes());
                    Ok(None)
                }
            }
        })?
    }
}
