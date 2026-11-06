use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationEncodedScalarValue, ApplicationEntityRef, ApplicationGraphMemberMeaning,
    ApplicationRelationRef,
};

use super::ApplicationQueryRootPathGuard;

pub use worth_foundational::facade::RelationDirection;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryRootPathTraversal {
    OneHop,
    Closure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryRootPathSourceKind {
    Stored,
    Derived,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryRootPathStep {
    relation: String,
    from: ApplicationGraphMemberMeaning,
    to: ApplicationGraphMemberMeaning,
    source_kind: ApplicationQueryRootPathSourceKind,
    direction: RelationDirection,
    traversal: ApplicationQueryRootPathTraversal,
}

impl ApplicationQueryRootPathStep {
    pub fn from_untrusted_fields(
        relation: String,
        from: String,
        to: String,
        direction: RelationDirection,
        traversal: ApplicationQueryRootPathTraversal,
    ) -> Self {
        Self {
            relation,
            from: from.into(),
            to: to.into(),
            source_kind: ApplicationQueryRootPathSourceKind::Stored,
            direction,
            traversal,
        }
    }

    pub fn from_untrusted_source_fields(
        relation: String,
        from: ApplicationGraphMemberMeaning,
        to: ApplicationGraphMemberMeaning,
        source_kind: ApplicationQueryRootPathSourceKind,
        direction: RelationDirection,
        traversal: ApplicationQueryRootPathTraversal,
    ) -> Self {
        Self {
            relation,
            from,
            to,
            source_kind,
            direction,
            traversal,
        }
    }
    pub const fn source_kind(&self) -> ApplicationQueryRootPathSourceKind {
        self.source_kind
    }
    pub fn from_member(&self) -> &ApplicationGraphMemberMeaning {
        &self.from
    }
    pub fn to_member(&self) -> &ApplicationGraphMemberMeaning {
        &self.to
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        self.from.name()
    }

    pub fn to(&self) -> &str {
        self.to.name()
    }

    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }

    pub const fn traversal(&self) -> ApplicationQueryRootPathTraversal {
        self.traversal
    }

    pub fn parent_entity(&self) -> &str {
        match self.direction {
            RelationDirection::Forward => self.from(),
            RelationDirection::Reverse => self.to(),
        }
    }

    pub fn child_entity(&self) -> &str {
        match self.direction {
            RelationDirection::Forward => self.to(),
            RelationDirection::Reverse => self.from(),
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
    field_expectations: Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    _marker: PhantomData<fn() -> (Schema, Start, Current)>,
}

impl<Schema, Start> ApplicationQueryRootPath<Schema, Start, Start> {
    pub fn from(start: ApplicationEntityRef<Schema, Start>) -> Self {
        Self {
            start_entity: start.name(),
            current_entity: start.name(),
            steps: Vec::new(),
            guards: Vec::new(),
            field_expectations: Vec::new(),
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
            from: relation.from().into(),
            to: relation.to().into(),
            source_kind: ApplicationQueryRootPathSourceKind::Stored,
            direction: RelationDirection::Forward,
            traversal: ApplicationQueryRootPathTraversal::OneHop,
        });
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.to(),
            steps: self.steps,
            guards: self.guards,
            field_expectations: self.field_expectations,
            _marker: PhantomData,
        }
    }

    pub fn reverse<Relation, Previous>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Previous, Current>,
    ) -> ApplicationQueryRootPath<Schema, Start, Previous> {
        self.steps.push(ApplicationQueryRootPathStep {
            relation: relation.name().to_owned(),
            from: relation.from().into(),
            to: relation.to().into(),
            source_kind: ApplicationQueryRootPathSourceKind::Stored,
            direction: RelationDirection::Reverse,
            traversal: ApplicationQueryRootPathTraversal::OneHop,
        });
        ApplicationQueryRootPath {
            start_entity: self.start_entity,
            current_entity: relation.from(),
            steps: self.steps,
            guards: self.guards,
            field_expectations: self.field_expectations,
            _marker: PhantomData,
        }
    }

    /// Reflexive complete reachability over one endorelation, including roots.
    pub fn forward_closure<Relation>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Current>,
    ) -> Self {
        self.steps
            .push(ApplicationQueryRootPathStep::from_untrusted_fields(
                relation.name().to_owned(),
                relation.from().to_owned(),
                relation.to().to_owned(),
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::Closure,
            ));
        self
    }

    /// Complete reverse reachability over one endorelation, including roots.
    pub fn reverse_closure<Relation>(
        mut self,
        relation: ApplicationRelationRef<Schema, Relation, Current, Current>,
    ) -> Self {
        self.steps
            .push(ApplicationQueryRootPathStep::from_untrusted_fields(
                relation.name().to_owned(),
                relation.from().to_owned(),
                relation.to().to_owned(),
                RelationDirection::Reverse,
                ApplicationQueryRootPathTraversal::Closure,
            ));
        self
    }

    pub fn where_equal<Field>(
        mut self,
        field: Field,
        expected: ApplicationEncodedScalarValue<Field::Binding>,
    ) -> Result<Self, crate::application_schema::ApplicationSchemaDeclarationDenial>
    where
        Field: crate::application_schema::fragment::RecordSelection<Entity = Current>,
    {
        field.checked()?;
        self.field_expectations.push(field.expected_recipe());
        self.guards.push(ApplicationQueryRootPathGuard::new(
            self.steps.len(),
            field,
            expected,
        ));
        Ok(self)
    }

    pub(crate) fn into_meaning(self) -> ApplicationQueryRootPathMeaning {
        self.into_parts().0
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ApplicationQueryRootPathMeaning,
        Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    ) {
        (
            ApplicationQueryRootPathMeaning {
                start_entity: self.start_entity.to_owned(),
                terminal_entity: self.current_entity.to_owned(),
                steps: self.steps,
                guards: self.guards,
            },
            self.field_expectations,
        )
    }
}

mod derived;
