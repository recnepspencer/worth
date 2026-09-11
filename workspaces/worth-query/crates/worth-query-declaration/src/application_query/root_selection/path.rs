use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationRelationRef, DeclaredApplicationFieldValue, EqualityCapable, EqualityPredicate,
    WritePosture,
};

use super::ApplicationQueryRootPathGuard;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryRootPathDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRootPathStep {
    relation: String,
    from: String,
    to: String,
    direction: ApplicationQueryRootPathDirection,
}

impl ApplicationQueryRootPathStep {
    pub fn from_untrusted_fields(
        relation: String,
        from: String,
        to: String,
        direction: ApplicationQueryRootPathDirection,
    ) -> Self {
        Self {
            relation,
            from,
            to,
            direction,
        }
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }

    pub const fn direction(&self) -> ApplicationQueryRootPathDirection {
        self.direction
    }

    pub fn parent_entity(&self) -> &str {
        match self.direction {
            ApplicationQueryRootPathDirection::Forward => &self.from,
            ApplicationQueryRootPathDirection::Reverse => &self.to,
        }
    }

    pub fn child_entity(&self) -> &str {
        match self.direction {
            ApplicationQueryRootPathDirection::Forward => &self.to,
            ApplicationQueryRootPathDirection::Reverse => &self.from,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRootPathMeaning {
    start_entity: String,
    terminal_entity: String,
    steps: Vec<ApplicationQueryRootPathStep>,
    guards: Vec<ApplicationQueryRootPathGuard>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableApplicationQueryRootPathParts {
    pub start_entity: String,
    pub terminal_entity: String,
    pub steps: Vec<ApplicationQueryRootPathStep>,
    pub guards: Vec<ApplicationQueryRootPathGuard>,
}

impl ApplicationQueryRootPathMeaning {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryRootPathParts) -> Self {
        Self {
            start_entity: parts.start_entity,
            terminal_entity: parts.terminal_entity,
            steps: parts.steps,
            guards: parts.guards,
        }
    }

    pub fn start_entity(&self) -> &str {
        &self.start_entity
    }

    pub fn terminal_entity(&self) -> &str {
        &self.terminal_entity
    }

    pub fn steps(&self) -> &[ApplicationQueryRootPathStep] {
        &self.steps
    }

    pub fn guards(&self) -> &[ApplicationQueryRootPathGuard] {
        &self.guards
    }
}

pub struct ApplicationQueryRootPath<Schema, Start, Current> {
    start_entity: &'static str,
    current_entity: &'static str,
    steps: Vec<ApplicationQueryRootPathStep>,
    guards: Vec<ApplicationQueryRootPathGuard>,
    _marker: PhantomData<fn() -> (Schema, Start, Current)>,
}

impl<Schema, Start> ApplicationQueryRootPath<Schema, Start, Start> {
    pub fn from(start: ApplicationEntityRef<Schema, Start>) -> Self {
        Self {
            start_entity: start.name(),
            current_entity: start.name(),
            steps: Vec::new(),
            guards: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Start, Current> ApplicationQueryRootPath<Schema, Start, Current> {
    pub fn forward<Relation, Next>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Next>,
    ) -> ApplicationQueryRootPath<Schema, Start, Next> {
        self.steps.push(ApplicationQueryRootPathStep {
            relation: relation.name().to_owned(),
            from: relation.from().to_owned(),
            to: relation.to().to_owned(),
            direction: ApplicationQueryRootPathDirection::Forward,
        });
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.to(),
            steps: self.steps,
            guards: self.guards,
            _marker: PhantomData,
        }
    }

    pub fn reverse<Relation, Previous>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Previous, Current>,
    ) -> ApplicationQueryRootPath<Schema, Start, Previous> {
        self.steps.push(ApplicationQueryRootPathStep {
            relation: relation.name().to_owned(),
            from: relation.from().to_owned(),
            to: relation.to().to_owned(),
            direction: ApplicationQueryRootPathDirection::Reverse,
        });
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.from(),
            steps: self.steps,
            guards: self.guards,
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
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Self
    where
        Field: DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        EqualityPredicate: EqualityCapable,
        Unit: ApplicationFieldUnit,
    {
        self.guards.push(ApplicationQueryRootPathGuard::new(
            self.steps.len(),
            field,
            expected,
        ));
        self
    }

    pub(crate) fn into_meaning(self) -> ApplicationQueryRootPathMeaning {
        ApplicationQueryRootPathMeaning {
            start_entity: self.start_entity.to_owned(),
            terminal_entity: self.current_entity.to_owned(),
            steps: self.steps,
            guards: self.guards,
        }
    }
}
