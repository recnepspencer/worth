//! The names a set of entity records carry at one observation, read in one
//! pass and admitted as kind-typed names on request.
//!
//! Every entry is a key Relational's index recorded from a canonical
//! committed birth, with the subject its installed scheme governs. Typing an
//! entry as `WorthQueryName<Entity>` verifies that subject against `Entity`.

use std::collections::BTreeMap;

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    LineageRecordTarget, PersistentNameKey, PersistentNameSubjectKind,
};

use crate::tracked_resolution::ExecutionNameReadDenial;

/// What one observed entity record carries.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Observed {
    /// No name was ever bound to the record.
    Unnamed,
    /// The record's name no longer designates it at the basis.
    Ended,
    /// The record is live under `digest`, born under scheme `scheme`.
    Named { scheme: usize, digest: [u8; 32] },
}

/// The persistent names of the entity records one read observed.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObservedEntityNames {
    records: BTreeMap<EntityId, Observed>,
    /// `(scheme id, installed subject name)` of each scheme a name was born
    /// under; every one governs an entity subject.
    schemes: Vec<(String, String)>,
}

impl ObservedEntityNames {
    pub(crate) fn record_unnamed(&mut self, entity: EntityId) {
        self.records.insert(entity, Observed::Unnamed);
    }

    pub(crate) fn record_ended(&mut self, entity: EntityId) {
        self.records.insert(entity, Observed::Ended);
    }

    /// Records the live name of `entity`, born under an installed scheme
    /// governing the entity subject `subject`.
    pub(crate) fn record_named(
        &mut self,
        entity: EntityId,
        key: PersistentNameKey,
        scheme_id: &str,
        subject: String,
    ) {
        let scheme = self
            .schemes
            .iter()
            .position(|(id, _)| id == scheme_id)
            .unwrap_or_else(|| {
                self.schemes.push((scheme_id.to_owned(), subject));
                self.schemes.len() - 1
            });
        let digest = *key.digest();
        self.records
            .insert(entity, Observed::Named { scheme, digest });
    }

    /// How many records the read observed.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Heap bytes this collection retains for its records and scheme names.
    pub fn retained_bytes(&self) -> usize {
        let record = std::mem::size_of::<EntityId>() + std::mem::size_of::<Observed>();
        self.schemes.iter().fold(
            self.records.len().saturating_mul(record).saturating_add(
                self.schemes
                    .capacity()
                    .saturating_mul(std::mem::size_of::<(String, String)>()),
            ),
            |bytes, (scheme, subject)| {
                bytes
                    .saturating_add(scheme.capacity())
                    .saturating_add(subject.capacity())
            },
        )
    }

    /// The admitted `WorthQueryName<Entity>` the observed `entity` carries;
    /// `None` when it carries no name.
    ///
    /// Refuses a record this read did not observe, a record whose name no
    /// longer designates it, and a name whose installed scheme governs
    /// another subject than `Entity`.
    ///
    /// Complexity: O(log observed records).
    pub fn name_of<Schema, Entity>(
        &self,
        entity: EntityId,
    ) -> Result<Option<WorthQueryName<Entity>>, ExecutionNameReadDenial>
    where
        Entity: ApplicationEntityMarkerIdentity<Schema>,
    {
        let record = LineageRecordTarget::Entity(entity);
        match self.records.get(&entity) {
            None => Err(ExecutionNameReadDenial::RecordNotObserved { record }),
            Some(Observed::Unnamed) => Ok(None),
            Some(Observed::Ended) => Err(ExecutionNameReadDenial::RecordNoLongerNamed { record }),
            Some(Observed::Named { scheme, digest }) => {
                let (scheme_id, subject) = &self.schemes[*scheme];
                if subject != Entity::IDENTIFIER {
                    return Err(ExecutionNameReadDenial::KindIdentityMismatch {
                        scheme_id: scheme_id.clone(),
                        expected_subject: subject.clone(),
                        requested_subject: Entity::IDENTIFIER.to_owned(),
                    });
                }
                Ok(Some(admit_minted_digest(*digest)))
            }
        }
    }
}

/// The entity subject an installed scheme governs, or the typed refusal.
pub(crate) fn entity_scheme_subject(
    scheme_id: Option<&str>,
    installed: Option<(PersistentNameSubjectKind, String)>,
) -> Result<(&str, String), ExecutionNameReadDenial> {
    let (Some(scheme_id), Some((kind, subject))) = (scheme_id, installed) else {
        return Err(ExecutionNameReadDenial::UnknownScheme {
            scheme_id: scheme_id.map(str::to_owned),
        });
    };
    if kind != PersistentNameSubjectKind::Entity {
        return Err(ExecutionNameReadDenial::SubjectKindMismatch {
            expected: PersistentNameSubjectKind::Entity,
            actual: kind,
        });
    }
    Ok((scheme_id, subject))
}
