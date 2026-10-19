use super::{ApplicationFieldValueMeaning, ApplicationValueMeaningDenial as D};
use crate::application_schema::ApplicationSchemaMember;
use std::collections::{BTreeMap, BTreeSet};
use worth_foundational::facade::{AspectContractRevision, AspectIdentity};

/// A sorted description of one aspect's declared value roster. Recovery may
/// carry this description; only the owning runtime can supply predecessor proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationAspectValueMeaning {
    entity: String,
    aspect: String,
    identity: AspectIdentity,
    revision: AspectContractRevision,
    absence: worth_foundational::facade::AbsenceLaw,
    fields: Vec<ApplicationFieldValueMeaning>,
}

impl ApplicationAspectValueMeaning {
    pub fn from_members(members: &[ApplicationSchemaMember]) -> Result<Vec<Self>, D> {
        let mut aspects = BTreeMap::new();
        let mut identities = BTreeSet::new();
        for member in members {
            let ApplicationSchemaMember::Aspect {
                entity,
                aspect,
                identity,
                revision,
                absence,
            } = member
            else {
                continue;
            };
            if revision.0 == 0 {
                return Err(D::RevisionZero);
            }
            if !identities.insert(*identity) {
                return Err(D::DuplicateAspectIdentity);
            }
            let meaning = Self {
                entity: entity.clone(),
                aspect: aspect.clone(),
                identity: *identity,
                revision: *revision,
                absence: *absence,
                fields: Vec::new(),
            };
            if aspects
                .insert((entity.as_str(), aspect.as_str()), meaning)
                .is_some()
            {
                return Err(D::DuplicateAspect);
            }
        }
        for member in members {
            let ApplicationSchemaMember::Field { entity, aspect, .. } = member else {
                continue;
            };
            let meaning = aspects
                .get_mut(&(entity.as_str(), aspect.as_str()))
                .ok_or(D::FieldWithoutAspect)?;
            meaning.fields.push(
                ApplicationFieldValueMeaning::from_member(member)?.expect("field member checked"),
            );
        }
        for meaning in aspects.values_mut() {
            meaning.fields.sort_by(|a, b| a.name().cmp(b.name()));
            if meaning
                .fields
                .windows(2)
                .any(|pair| pair[0].name() == pair[1].name())
            {
                return Err(D::DuplicateField);
            }
        }
        Ok(aspects.into_values().collect())
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn aspect(&self) -> &str {
        &self.aspect
    }
    pub const fn identity(&self) -> AspectIdentity {
        self.identity
    }
    pub const fn revision(&self) -> AspectContractRevision {
        self.revision
    }
    pub const fn absence(&self) -> worth_foundational::facade::AbsenceLaw {
        self.absence
    }
    pub fn fields(&self) -> &[ApplicationFieldValueMeaning] {
        &self.fields
    }
}
