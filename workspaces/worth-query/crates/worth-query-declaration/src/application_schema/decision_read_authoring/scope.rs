//! A typed scope owns exact read contents, independently of its Rust marker's name.
use super::super::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget as Target, ApplicationRelationRef,
    ApplicationSchemaDeclarationDenial as Denial, OperationReads,
};
use std::marker::PhantomData;

/// Content-addressed, authority-free description of one reusable decision-read scope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationDecisionReadScopeMeaning {
    targets: Vec<Target>,
    derived_relations: Vec<String>,
    identity: String,
}

impl ApplicationDecisionReadScopeMeaning {
    pub fn from_untrusted_parts(
        mut targets: Vec<Target>,
        mut derived_relations: Vec<String>,
    ) -> Result<Self, Denial> {
        targets.sort();
        targets.dedup();
        derived_relations.sort();
        derived_relations.dedup();
        let identity = super::identity::content_identity(&targets, &derived_relations)?;
        Ok(Self {
            targets,
            derived_relations,
            identity,
        })
    }

    pub fn targets(&self) -> &[Target] {
        &self.targets
    }
    pub fn derived_relations(&self) -> &[String] {
        &self.derived_relations
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// A scope marker receives exactly the typed grants its author puts in this definition.
pub struct ApplicationDecisionReadScopeDefinition<Schema, Scope> {
    pub(super) targets: Vec<Target>,
    pub(super) derived_relations: Vec<String>,
    pub(super) fields: Vec<super::super::ApplicationFieldBindingRecipe>,
    marker: PhantomData<fn() -> (Schema, Scope)>,
}

impl<S, Scope> Default for ApplicationDecisionReadScopeDefinition<S, Scope> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, Scope> ApplicationDecisionReadScopeDefinition<S, Scope> {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            derived_relations: Vec::new(),
            fields: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn entity<E: OperationReads<Scope>>(mut self, entity: ApplicationEntityRef<S, E>) -> Self {
        self.targets.push(Target::Entity {
            entity: entity.name().to_owned(),
        });
        self
    }

    pub fn field<F: super::super::fragment::RecordSelection>(mut self, field: F) -> Self {
        let recipe = field.expected_recipe();
        let locus = recipe.locus();
        self.targets.push(Target::Field {
            entity: locus.entity().to_owned(),
            aspect: locus.aspect().to_owned(),
            field: locus.field().to_owned(),
        });
        self.fields.push(recipe);
        self
    }

    pub fn relation<R: OperationReads<Scope>, From, To>(
        mut self,
        relation: ApplicationRelationRef<S, R, From, To>,
    ) -> Self {
        self.targets.push(Target::Relation {
            relation: relation.name().to_owned(),
            from: relation.from().to_owned(),
            to: relation.to().to_owned(),
        });
        self
    }

    pub fn derived_relation<R: OperationReads<Scope>, From, To>(
        mut self,
        relation: super::super::ApplicationDerivedRelationRef<S, R, From, To>,
    ) -> Self {
        self.derived_relations.push(relation.name().to_owned());
        self
    }

    pub fn managed_index<Q: crate::application_query::ApplicationQueryBinding<S>>(mut self) -> Self
    where
        S: super::super::ApplicationSchema,
    {
        self.targets.push(Target::ManagedIndex {
            parent: crate::application_query::ApplicationQueryBindingReference::of::<S, Q>(),
        });
        self
    }
}
