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
    scalar_field: Option<String>,
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
                scalar_field: None,
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
        for member in members {
            if let ApplicationSchemaMember::ScalarAspectField {
                entity,
                aspect,
                field,
            } = member
            {
                let meaning = aspects
                    .get_mut(&(entity.as_str(), aspect.as_str()))
                    .ok_or(D::FieldWithoutAspect)?;
                if meaning.scalar_field.replace(field.clone()).is_some()
                    || meaning.fields.len() != 1
                    || !meaning.fields.iter().any(|declared| {
                        declared.name() == field
                            && declared.presence()
                                == crate::application_schema::ApplicationFieldPresence::Required
                            && declared.codec_digest().is_none()
                    })
                {
                    return Err(D::InvalidScalarStorage);
                }
            }
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
    pub fn scalar_field(&self) -> Option<&str> {
        self.scalar_field.as_deref()
    }
}
