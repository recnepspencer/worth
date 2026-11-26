//! Name and birth reads a later member of an edit envelope makes over the
//! names its earlier members give their records (#426).
//!
//! The source is the envelope's own naming authority: committed names at
//! the envelope's basis overlaid with the names its composed naming plan
//! mints, keeps and ends over the applied state, exactly as finalization
//! records them. Every answer is admitted against the installed scheme that
//! governs it, committed or pending, so a name of another kind refuses
//! typed and a discriminator reference to one never decodes as this kind.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationEntityMarkerIdentity,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{LineageRecordTarget, PersistentNameSubjectKind};
use worth_relational::facade::runtime::{
    CustomInvariantNamePosture, CustomInvariantNameReadDenial, CustomInvariantNameSource,
};

use crate::admission::{admit_birth_of, InCandidate, RequestedSubject};
use crate::invariant_reads::{entity, name_in, InvariantNameRead};
use crate::tracked_resolution::ExecutionNameReadDenial;

/// A recorded birth, and the record the envelope mints it for: `None` for a
/// birth committed at the basis.
pub type CandidateBirth<K> = (RecordedNameBirth<K>, Option<LineageRecordTarget>);

const fn relation<Relation: fragment::Relation>() -> RequestedSubject<'static> {
    RequestedSubject {
        kind: PersistentNameSubjectKind::Relation,
        name: Relation::NAME,
    }
}

/// The admitted `WorthQueryName<Entity>` the candidate gives `entity_id`.
/// Refuses typed when the envelope gives it none, ended its name, or named
/// it under another kind's scheme; the outer refusal is the envelope's
/// naming authority (a refused plan, or a stale naming basis).
///
/// Complexity: O(log r) in the envelope's r named records once its preview
/// ran; the first read runs it.
pub fn candidate_entity_name_of<Schema, Entity>(
    source: &dyn CustomInvariantNameSource,
    entity_id: EntityId,
) -> InvariantNameRead<WorthQueryName<Entity>>
where
    Entity: ApplicationEntityMarkerIdentity<Schema>,
{
    let record = LineageRecordTarget::Entity(entity_id);
    proposed(source, |names| {
        let carried = names.view.record_name(record);
        name_in(names, carried, record, entity::<Schema, Entity>())
    })
}

/// The admitted `WorthQueryName<Relation>` the candidate gives `relation_id`.
///
/// Complexity: as [`candidate_entity_name_of`].
pub fn candidate_relation_name_of<Relation: fragment::Relation>(
    source: &dyn CustomInvariantNameSource,
    relation_id: RelationId,
) -> InvariantNameRead<WorthQueryName<Relation>> {
    let record = LineageRecordTarget::Relation(relation_id);
    proposed(source, |names| {
        let carried = names.view.record_name(record);
        name_in(names, carried, record, relation::<Relation>())
    })
}

/// The recorded birth of a held entity name, committed or minted by the
/// envelope, decoded with [`RecordedNameBirth::decode`]. Refuses
/// `UnknownName` for a name neither records.
///
/// Complexity: O(1) expected probes per parent and per referenced name, plus
/// O(B) for the B discriminator bytes.
pub fn candidate_entity_birth_of<Schema, Entity>(
    source: &dyn CustomInvariantNameSource,
    held: &WorthQueryName<Entity>,
) -> InvariantNameRead<CandidateBirth<Entity>>
where
    Entity: ApplicationEntityMarkerIdentity<Schema>,
{
    birth(source, held.as_bytes(), entity::<Schema, Entity>())
}

/// The recorded birth of a held relation name, committed or minted.
///
/// Complexity: as [`candidate_entity_birth_of`].
pub fn candidate_relation_birth_of<Relation: fragment::Relation>(
    source: &dyn CustomInvariantNameSource,
    held: &WorthQueryName<Relation>,
) -> InvariantNameRead<CandidateBirth<Relation>> {
    birth(source, held.as_bytes(), relation::<Relation>())
}

fn birth<K>(
    source: &dyn CustomInvariantNameSource,
    digest: &[u8; 32],
    requested: RequestedSubject<'_>,
) -> InvariantNameRead<CandidateBirth<K>> {
    proposed(source, |names| {
        let key = names
            .view
            .recorded_name(requested.kind, digest)
            .ok_or(ExecutionNameReadDenial::UnknownName)?;
        let birth = admit_birth_of(names, key, requested)?;
        Ok((birth, names.view.born_record(key)))
    })
}

/// Runs `read` over the source's proposed posture.
fn proposed<T>(
    source: &dyn CustomInvariantNameSource,
    read: impl FnOnce(&InCandidate<'_>) -> T,
) -> Result<T, CustomInvariantNameReadDenial> {
    let mut read = Some(read);
    let mut answer = None;
    source.read(CustomInvariantNamePosture::Proposed, &mut |view| {
        answer = read.take().map(|read| read(&InCandidate { source, view }));
    })?;
    answer.ok_or(CustomInvariantNameReadDenial::Unavailable)
}
