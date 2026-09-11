use std::marker::PhantomData;

use worth_foundational::facade::AspectValue;

use super::capabilities::{ApplicationFieldUnit, EqualityCapable, EqualityPredicate};
use super::field_reference::ApplicationFieldRef;
use super::references::{ApplicationEntityRef, ApplicationRelationRef};
use super::values::DeclaredApplicationFieldValue;
use super::ApplicationEncodedScalarValue;

mod portable_parts;
pub use portable_parts::{
    WorthQueryPortableApplicationAuthorizationPathParts,
    WorthQueryPortableApplicationAuthorizationPredicateParts,
    WorthQueryPortableApplicationAuthorizationTraversalParts,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationAuthorizationPathEffect {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationAuthorizationTraversalDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationTraversal {
    relation: String,
    from: String,
    to: String,
    direction: ApplicationAuthorizationTraversalDirection,
}

impl ApplicationAuthorizationTraversal {
    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> ApplicationAuthorizationTraversalDirection {
        self.direction
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationPredicate {
    traversal_ordinal: usize,
    entity: String,
    aspect: String,
    field: String,
    value: AspectValue,
}

impl ApplicationAuthorizationPredicate {
    pub const fn traversal_ordinal(&self) -> usize {
        self.traversal_ordinal
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &str {
        &self.aspect
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn value(&self) -> &AspectValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationAuthorizationPath {
    effect: ApplicationAuthorizationPathEffect,
    principal_entity: String,
    scope_entity: String,
    traversals: Vec<ApplicationAuthorizationTraversal>,
    predicates: Vec<ApplicationAuthorizationPredicate>,
}

impl ApplicationAuthorizationPath {
    pub const fn effect(&self) -> ApplicationAuthorizationPathEffect {
        self.effect
    }

    pub fn principal_entity(&self) -> &str {
        &self.principal_entity
    }

    pub fn scope_entity(&self) -> &str {
        &self.scope_entity
    }

    pub fn traversals(&self) -> &[ApplicationAuthorizationTraversal] {
        &self.traversals
    }

    pub fn predicates(&self) -> &[ApplicationAuthorizationPredicate] {
        &self.predicates
    }
}

pub struct ApplicationAuthorizationPathBuilder<Schema, Current> {
    principal_entity: &'static str,
    current_entity: &'static str,
    traversals: Vec<ApplicationAuthorizationTraversal>,
    predicates: Vec<ApplicationAuthorizationPredicate>,
    _marker: PhantomData<fn() -> (Schema, Current)>,
}

impl<Schema, Principal> ApplicationAuthorizationPathBuilder<Schema, Principal> {
    pub fn from_principal(principal: ApplicationEntityRef<Schema, Principal>) -> Self {
        Self {
            principal_entity: principal.name(),
            current_entity: principal.name(),
            traversals: Vec::new(),
            predicates: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Current> ApplicationAuthorizationPathBuilder<Schema, Current> {
    pub fn forward<Relation, Next>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Next>,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Next> {
        self.traversals.push(ApplicationAuthorizationTraversal {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
            direction: ApplicationAuthorizationTraversalDirection::Forward,
        });
        ApplicationAuthorizationPathBuilder {
            principal_entity: self.principal_entity,
            current_entity: relation.to(),
            traversals: self.traversals,
            predicates: self.predicates,
            _marker: PhantomData,
        }
    }

    pub fn reverse<Relation, Previous>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Previous, Current>,
    ) -> ApplicationAuthorizationPathBuilder<Schema, Previous> {
        self.traversals.push(ApplicationAuthorizationTraversal {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
            direction: ApplicationAuthorizationTraversalDirection::Reverse,
        });
        ApplicationAuthorizationPathBuilder {
            principal_entity: self.principal_entity,
            current_entity: relation.from(),
            traversals: self.traversals,
            predicates: self.predicates,
            _marker: PhantomData,
        }
    }

    pub fn where_equal<Aspect, Field, Value, Write, Unit>(
        mut self,
        field: ApplicationFieldRef<
            Schema,
            Current,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
        EqualityPredicate: EqualityCapable,
    {
        self.predicates.push(ApplicationAuthorizationPredicate {
            traversal_ordinal: self.traversals.len(),
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
            value: value.into_foundational_value(),
        });
        self
    }

    pub fn allow(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPath {
        self.finish(scope, ApplicationAuthorizationPathEffect::Allow)
    }

    pub fn deny(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
    ) -> ApplicationAuthorizationPath {
        self.finish(scope, ApplicationAuthorizationPathEffect::Deny)
    }

    fn finish(
        self,
        scope: ApplicationEntityRef<Schema, Current>,
        effect: ApplicationAuthorizationPathEffect,
    ) -> ApplicationAuthorizationPath {
        debug_assert_eq!(self.current_entity, scope.name());
        ApplicationAuthorizationPath {
            effect,
            principal_entity: self.principal_entity.to_string(),
            scope_entity: scope.name().to_string(),
            traversals: self.traversals,
            predicates: self.predicates,
        }
    }
}
