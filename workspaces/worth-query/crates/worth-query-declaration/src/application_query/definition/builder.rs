use std::marker::PhantomData;
mod managed_index;

use crate::application_query::ApplicationQueryMarkerIdentity;
use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryContinuationTarget,
    ApplicationQueryDefinitionDenial, ApplicationQueryLiveCauseBinding,
    ApplicationQueryLiveCauseContract, ApplicationQueryOrderingDirection,
    ApplicationQueryOrderingTerm, ApplicationQueryParameterDefinition,
    ApplicationQueryParameterRef, ApplicationQueryReference, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultTraversal, ApplicationQueryRootPath,
    ManyResults,
};

use crate::portable_identity::WorthQueryPortableType;

use super::authoring::{ApplicationQueryDefinitionParts, ApplicationQueryRootAuthoring};
use super::{ApplicationQueryDefinition, ApplicationQueryPredicate};

pub struct ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope> {
    definition: ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope>
{
    pub fn declare(
        reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> ApplicationQueryRootAuthoring<Schema, Query, Parameters, QueryResult, Scope> {
        ApplicationQueryRootAuthoring::new(reference)
    }

    pub(super) fn with_authorization<Root>(
        parts: ApplicationQueryDefinitionParts<Schema, Query, Parameters, QueryResult, Scope, Root>,
        authorization: ApplicationQueryAuthorizationRequirement,
    ) -> Self {
        Self {
            definition: ApplicationQueryDefinition {
                name: parts.reference.name(),
                query_type: parts.reference.query_type(),
                parameter_type: parts.reference.parameter_type(),
                result_type: parts.reference.result_type(),
                scope_type: parts.reference.scope_type(),
                root_entity: parts.root.name(),
                scope_entity: parts.scope.name(),
                parameters: Vec::new(),
                result_shape: parts.result_shape,
                field_expectations: parts.field_expectations,
                root_paths: Vec::new(),
                orbit: None,
                component_selection: None,
                managed_index: None,
                managed_index_recipe: None,
                cardinality: parts.cardinality,
                predicates: Vec::new(),
                ordering: Vec::new(),
                continuation: None,
                live_cause: None,
                disclosure: parts.disclosure,
                authorization,
                basis_support: parts.basis_support,
                lanes: parts.lanes,
                _marker: PhantomData,
            },
        }
    }

    pub fn parameter<Parameter, Binding>(
        mut self,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Binding: crate::application_schema::ApplicationScalarValueBinding,
    {
        self.definition
            .parameters
            .push(ApplicationQueryParameterDefinition::typed(parameter));
        self
    }

    pub fn root_path<Root>(mut self, path: ApplicationQueryRootPath<Schema, Scope, Root>) -> Self {
        let (meaning, fields) = path.into_parts();
        self.definition.field_expectations.extend(fields);
        self.definition.root_paths.push(meaning);
        self
    }

    /// Select one complete stored orbit from the admitted scope member.
    /// It cannot be combined with path unions, predicates, ordering or paging.
    pub fn orbit<Slot, Relation>(
        mut self,
        orbit: super::super::ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Scope>,
    ) -> Self {
        self.definition.orbit = Some(orbit.into_meaning());
        self
    }

    /// Select complete Native components through this query's explicit root-path union.
    pub fn component_selection<Slot, Set, Entity>(
        mut self,
        selection: super::super::ApplicationQueryComponentRef<Schema, Query, Slot, Set, Entity>,
    ) -> Self {
        self.definition.component_selection = Some(selection.into_meaning());
        self
    }

    pub fn where_equal<Field, Parameter, Binding>(
        mut self,
        field: Field,
        parameter: ApplicationQueryParameterRef<Query, Parameter, Binding>,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<Binding = Binding>,
        Binding: crate::application_schema::ApplicationReadableScalarValueBinding,
    {
        self.definition
            .field_expectations
            .push(field.expected_recipe());
        self.definition.predicates.push(ApplicationQueryPredicate {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
            parameter: parameter.name().to_owned(),
            scalar_family: Binding::SCALAR_FAMILY,
        });
        self
    }

    pub fn order_by<Slot, Entity, Aspect, Field, Value>(
        mut self,
        field: ApplicationQueryResultFieldRef<Query, Slot, Schema, Field>,
        direction: ApplicationQueryOrderingDirection,
    ) -> Self
    where
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.definition
            .ordering
            .push(ApplicationQueryOrderingTerm::from_result_field(
                field, direction,
            ));
        self
    }

    pub fn continue_by<Slot, Relation, From, To, Direction>(
        mut self,
        relation: ApplicationQueryResultRelationRef<
            Query,
            Slot,
            Schema,
            Relation,
            From,
            To,
            Direction,
            ManyResults,
        >,
    ) -> Self
    where
        Direction: ApplicationQueryResultTraversal,
        Query: ApplicationQueryMarkerIdentity<Schema>,
        Slot: WorthQueryPortableType,
    {
        self.definition.continuation = Some(
            ApplicationQueryContinuationTarget::from_many_relation(relation),
        );
        self
    }

    pub fn live_by<Target, Binding>(
        mut self,
        scope_identity: ApplicationQueryResultFieldRef<Query, impl WorthQueryPortableType, Schema,
            impl crate::application_schema::fragment::RecordSelection<Entity = Scope,
                Value = <Binding::ScopeIdentityBinding as crate::application_schema::ApplicationScalarValueBinding>::Value,
                Binding = Binding::ScopeIdentityBinding>>,
        target_identity: ApplicationQueryResultFieldRef<Query, impl WorthQueryPortableType, Schema,
            impl crate::application_schema::fragment::RecordSelection<Entity = Target,
                Value = <Binding::TargetIdentityBinding as crate::application_schema::ApplicationScalarValueBinding>::Value,
                Binding = Binding::TargetIdentityBinding>>,
    ) -> Self
    where
        Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Target>,
        Query: ApplicationQueryMarkerIdentity<Schema>,
    {
        self.definition
            .field_expectations
            .push(scope_identity.binding_recipe());
        self.definition
            .field_expectations
            .push(target_identity.binding_recipe());
        self.definition.live_cause = Some(ApplicationQueryLiveCauseContract::typed(
            scope_identity,
            target_identity,
            super::super::ApplicationQueryLiveTargetMode::Collection,
            std::marker::PhantomData::<Binding>,
        ));
        self
    }

    pub fn live_root_by<Binding>(
        mut self,
        scope_identity: ApplicationQueryResultFieldRef<Query, impl WorthQueryPortableType, Schema,
            impl crate::application_schema::fragment::RecordSelection<Entity = Scope,
                Value = <Binding::ScopeIdentityBinding as crate::application_schema::ApplicationScalarValueBinding>::Value,
                Binding = Binding::ScopeIdentityBinding>>,
        target_identity: ApplicationQueryResultFieldRef<Query, impl WorthQueryPortableType, Schema,
            impl crate::application_schema::fragment::RecordSelection<Entity = Scope,
                Value = <Binding::TargetIdentityBinding as crate::application_schema::ApplicationScalarValueBinding>::Value,
                Binding = Binding::TargetIdentityBinding>>,
    ) -> Self
    where
        Binding: ApplicationQueryLiveCauseBinding<Schema, Query, Scope, Scope>,
        Query: ApplicationQueryMarkerIdentity<Schema>,
    {
        self.definition
            .field_expectations
            .push(scope_identity.binding_recipe());
        self.definition
            .field_expectations
            .push(target_identity.binding_recipe());
        self.definition.live_cause = Some(ApplicationQueryLiveCauseContract::typed(
            scope_identity,
            target_identity,
            super::super::ApplicationQueryLiveTargetMode::Root,
            std::marker::PhantomData::<Binding>,
        ));
        self
    }

    /// Projects unvalidated descriptive meaning. Schema registration remains the admission boundary.
    pub fn into_erased(mut self) -> super::super::ErasedApplicationQueryDefinition {
        self.normalize();
        self.definition.into_erased()
    }

    fn normalize(&mut self) {
        self.definition.parameters.sort();
        self.definition.predicates.sort();
        self.definition.root_paths.sort();
        self.definition.root_paths.dedup();
    }

    pub(crate) fn name(&self) -> &'static str {
        self.definition.name()
    }

    pub(crate) fn build(
        mut self,
    ) -> Result<
        ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
        ApplicationQueryDefinitionDenial,
    > {
        self.normalize();
        let portable =
            super::super::WorthQueryPortableApplicationQueryParts::project_typed(&self.definition);
        crate::application_query::validate_portable_application_query_freshly(&portable)?;
        Ok(self.definition)
    }
}
