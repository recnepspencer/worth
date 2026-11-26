//! Name and birth reads an installed invariant makes over the commit under
//! evaluation, through the same kind and birth admission tracked readers use.
//!
//! The invariant's posture decides what is recorded: the committed posture
//! reads the branch's names at the proposal's basis; the proposed posture
//! also reads the names the commit itself mints, keeps and ends. Every answer
//! is verified against the installed scheme that governs it, so a name of
//! another kind refuses typed and never decodes.

use worth_query_declaration::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{LineageRecordTarget, PersistentNameSubjectKind};
use worth_relational::facade::runtime::{
    CustomInvariantNameReadDenial, CustomInvariantNameReads, CustomInvariantRecordName,
};

use crate::admission::{
    admit_birth_of, kind_verified_resolved_name, InInvariant, RequestedSubject,
};
use crate::tracked_resolution::ExecutionNameReadDenial;

/// The outer refusal is the invariant's name source; the inner one the read.
pub type InvariantNameRead<T> =
    Result<Result<T, ExecutionNameReadDenial>, CustomInvariantNameReadDenial>;

const fn entity<Schema, Entity: ApplicationEntityMarkerIdentity<Schema>>(
) -> RequestedSubject<'static> {
    RequestedSubject {
        kind: PersistentNameSubjectKind::Entity,
        name: Entity::IDENTIFIER,
    }
}

/// The admitted `WorthQueryName<Entity>` `entity` carries in `names`'
/// posture. Refuses typed when it carries none, when its name ended in that
/// posture, or when its name's scheme governs another kind.
///
/// Complexity: O(1) expected index probes, or O(log r) in the commit's r
/// named records.
pub fn invariant_entity_name_of<Schema, Entity>(
    names: &CustomInvariantNameReads<'_>,
    entity_id: EntityId,
) -> InvariantNameRead<WorthQueryName<Entity>>
where
    Entity: ApplicationEntityMarkerIdentity<Schema>,
{
    let record = LineageRecordTarget::Entity(entity_id);
    names.read(|view| match view.record_name(record) {
        CustomInvariantRecordName::Named(key) => kind_verified_resolved_name(
            &InInvariant { names, view },
            key,
            entity::<Schema, Entity>(),
        ),
        CustomInvariantRecordName::Ended => {
            Err(ExecutionNameReadDenial::RecordNoLongerNamed { record })
        }
        CustomInvariantRecordName::Unnamed => {
            Err(ExecutionNameReadDenial::RecordNotNamed { record })
        }
    })
}

/// The recorded birth of a held entity name in `names`' posture, decoded
/// with [`RecordedNameBirth::decode`]. A name the commit mints answers with
/// the exact parts finalization records. Refuses `UnknownName` for a name
/// not recorded in that posture.
///
/// Complexity: O(1) expected probes per parent and per referenced name, plus
/// O(B) for the B discriminator bytes.
pub fn invariant_entity_birth_of<Schema, Entity>(
    names: &CustomInvariantNameReads<'_>,
    held: &WorthQueryName<Entity>,
) -> InvariantNameRead<RecordedNameBirth<Entity>>
where
    Entity: ApplicationEntityMarkerIdentity<Schema>,
{
    let requested = entity::<Schema, Entity>();
    names.read(|view| {
        let key = view
            .recorded_name(requested.kind, held.as_bytes())
            .ok_or(ExecutionNameReadDenial::UnknownName)?;
        admit_birth_of(&InInvariant { names, view }, key, requested)
    })
}
