//! Descriptive component-set meaning; installation owns its graph binding.
use std::marker::PhantomData;

use super::{ApplicationEntityRef, ApplicationRelationRef};
mod validation;
pub(super) use validation::{validate_closure, validate_identifiers};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationComponentSetMeaning {
    name: String,
    member: String,
    relations: Vec<String>,
}

/// Untrusted portable description. Fresh schema validation remains mandatory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationComponentSetParts {
    pub name: String,
    pub member: String,
    pub relations: Vec<String>,
}

impl ApplicationComponentSetMeaning {
    pub fn from_untrusted_parts(mut parts: WorthQueryPortableApplicationComponentSetParts) -> Self {
        parts.relations.sort();
        parts.relations.dedup();
        Self {
            name: parts.name,
            member: parts.member,
            relations: parts.relations,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn member(&self) -> &str {
        &self.member
    }
    pub fn relations(&self) -> &[String] {
        &self.relations
    }
}

/// A name selected by a schema, never permission to inspect its members.
pub struct ApplicationComponentSetRef<Schema, Set, Entity> {
    name: &'static str,
    member: &'static str,
    _marker: PhantomData<fn() -> (Schema, Set, Entity)>,
}
impl<Schema, Set, Entity> Copy for ApplicationComponentSetRef<Schema, Set, Entity> {}
impl<Schema, Set, Entity> Clone for ApplicationComponentSetRef<Schema, Set, Entity> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Schema, Set, Entity> ApplicationComponentSetRef<Schema, Set, Entity> {
    pub const fn new(name: &'static str, member: ApplicationEntityRef<Schema, Entity>) -> Self {
        Self::from_schema_identifiers(name, member.name())
    }
    #[doc(hidden)]
    pub const fn from_schema_identifiers(name: &'static str, member: &'static str) -> Self {
        Self {
            name,
            member,
            _marker: PhantomData,
        }
    }
    pub const fn name(&self) -> &'static str {
        self.name
    }
    pub const fn member(&self) -> &'static str {
        self.member
    }
}

/// Different relation markers may couple the same declared member type.
pub struct ApplicationComponentSetDefinition<Schema, Set, Entity> {
    reference: ApplicationComponentSetRef<Schema, Set, Entity>,
    relations: Vec<String>,
}
impl<Schema, Set, Entity> ApplicationComponentSetDefinition<Schema, Set, Entity> {
    pub fn new(reference: ApplicationComponentSetRef<Schema, Set, Entity>) -> Self {
        Self {
            reference,
            relations: Vec::new(),
        }
    }
    pub fn coupled_by<Relation>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
    ) -> Self {
        self.relations.push(relation.name().to_owned());
        self
    }
    pub(crate) fn into_meaning(self) -> ApplicationComponentSetMeaning {
        ApplicationComponentSetMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationComponentSetParts {
                name: self.reference.name.into(),
                member: self.reference.member.into(),
                relations: self.relations,
            },
        )
    }
}
