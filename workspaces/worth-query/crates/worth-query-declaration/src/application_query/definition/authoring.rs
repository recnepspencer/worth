use crate::application_query::ApplicationQueryMarkerIdentity;
use crate::application_query::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility, ApplicationQueryReference,
    ApplicationQueryResultShape, TypedApplicationQueryResultShape,
};
use crate::application_schema::{
    ApplicationAbilityRef, ApplicationEntityRef, ApplicationStructuredValueBinding,
};

use super::{
    ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDependencyCeiling,
};

pub struct ApplicationQueryRootAuthoring<Schema, Query, Parameters, QueryResult, Scope> {
    reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationQueryRootAuthoring<Schema, Query, Parameters, QueryResult, Scope>
{
    pub(super) const fn new(
        reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        Self { reference }
    }

    pub fn root<Root>(
        self,
        root: ApplicationEntityRef<Schema, Root>,
    ) -> ApplicationQueryScopeAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
        ApplicationQueryScopeAuthoring {
            reference: self.reference,
            root,
        }
    }
}

pub struct ApplicationQueryScopeAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
    reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    root: ApplicationEntityRef<Schema, Root>,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryScopeAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn scope(
        self,
        scope: ApplicationEntityRef<Schema, Scope>,
    ) -> ApplicationQueryResultAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
        ApplicationQueryResultAuthoring {
            reference: self.reference,
            root: self.root,
            scope,
        }
    }
}

pub struct ApplicationQueryResultAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
    reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    root: ApplicationEntityRef<Schema, Root>,
    scope: ApplicationEntityRef<Schema, Scope>,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryResultAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    /// Attaches the query marker's declared root result binding.
    ///
    /// A different binding for the same Rust value cannot replace the query's
    /// declared protocol identity:
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::{
    ///     application_query::{ApplicationQueryDefinitionBuilder, ApplicationQueryResultShapeBuilder},
    ///     application_schema::ApplicationEntityRef,
    /// };
    /// struct Schema;
    /// struct Parameters;
    /// struct Result;
    /// worth_query_declaration::worth_query_structured_value_binding!(ParametersBinding for Parameters { identity: "example.parameters.v1" });
    /// worth_query_declaration::worth_query_structured_value_binding!(DeclaredResultBinding for Result { identity: "example.result.v1" });
    /// worth_query_declaration::worth_query_structured_value_binding!(WrongResultBinding for Result { identity: "example.wrong-result.v1" });
    /// worth_query_declaration::worth_query_entity!(pub Record for Schema);
    /// worth_query_declaration::worth_query_application_query!(
    ///     pub ReadRecord for Schema,
    ///     identity "example.read-record.v1",
    ///     parameters ParametersBinding,
    ///     result DeclaredResultBinding,
    ///     scope Record => "Record",
    ///     name "read_record"
    /// );
    /// let root = ApplicationEntityRef::<Schema, Record>::from_schema_identifier("Record");
    /// let wrong = ApplicationQueryResultShapeBuilder::<
    ///     Schema, ReadRecord, Record, Result, WrongResultBinding,
    /// >::new(root).build();
    /// let _ = ApplicationQueryDefinitionBuilder::declare(ReadRecord::reference())
    ///     .root(root)
    ///     .scope(root)
    ///     .result_shape(wrong);
    /// ```
    pub fn result_shape<ShapeBinding>(
        self,
        result_shape: TypedApplicationQueryResultShape<
            Schema,
            Query,
            Root,
            QueryResult,
            ShapeBinding,
        >,
    ) -> ApplicationQueryCardinalityAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
    where
        Query: ApplicationQueryMarkerIdentity<Schema, ResultBinding = ShapeBinding>,
        ShapeBinding: ApplicationStructuredValueBinding<Value = QueryResult>,
    {
        ApplicationQueryCardinalityAuthoring {
            reference: self.reference,
            root: self.root,
            scope: self.scope,
            result_shape: result_shape.into_erased(),
        }
    }
}

pub struct ApplicationQueryCardinalityAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    root: ApplicationEntityRef<Schema, Root>,
    scope: ApplicationEntityRef<Schema, Scope>,
    result_shape: ApplicationQueryResultShape,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryCardinalityAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn cardinality(
        self,
        cardinality: ApplicationQueryCardinality,
    ) -> ApplicationQueryDependencyAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
    {
        ApplicationQueryDependencyAuthoring {
            previous: self,
            cardinality,
        }
    }
}

pub struct ApplicationQueryDependencyAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    previous:
        ApplicationQueryCardinalityAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>,
    cardinality: ApplicationQueryCardinality,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryDependencyAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn dependency_ceiling(
        self,
        dependency_ceiling: ApplicationQueryDependencyCeiling,
    ) -> ApplicationQueryDisclosureAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
    {
        ApplicationQueryDisclosureAuthoring {
            previous: self,
            dependency_ceiling,
        }
    }
}

pub struct ApplicationQueryDisclosureAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    previous:
        ApplicationQueryDependencyAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>,
    dependency_ceiling: ApplicationQueryDependencyCeiling,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryDisclosureAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn disclosure(
        self,
        disclosure: ApplicationQueryDisclosureContract,
    ) -> ApplicationQueryBasisAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
        ApplicationQueryBasisAuthoring {
            previous: self,
            disclosure,
        }
    }
}

pub struct ApplicationQueryBasisAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
    previous:
        ApplicationQueryDisclosureAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>,
    disclosure: ApplicationQueryDisclosureContract,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryBasisAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn basis_support(
        self,
        basis_support: ApplicationQueryBasisSupport,
    ) -> ApplicationQueryLaneAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
        ApplicationQueryLaneAuthoring {
            previous: self,
            basis_support,
        }
    }
}

pub struct ApplicationQueryLaneAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root> {
    previous: ApplicationQueryBasisAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>,
    basis_support: ApplicationQueryBasisSupport,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryLaneAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn lanes(
        self,
        lanes: ApplicationQueryLaneEligibility,
    ) -> ApplicationQueryAuthorizationAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
    {
        ApplicationQueryAuthorizationAuthoring {
            previous: self,
            lanes,
        }
    }
}

pub struct ApplicationQueryAuthorizationAuthoring<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Root,
> {
    previous: ApplicationQueryLaneAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>,
    lanes: ApplicationQueryLaneEligibility,
}

impl<Schema, Query, Parameters, QueryResult, Scope, Root>
    ApplicationQueryAuthorizationAuthoring<Schema, Query, Parameters, QueryResult, Scope, Root>
{
    pub fn public(
        self,
    ) -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope> {
        self.authorize(ApplicationQueryAuthorizationRequirement::public())
    }

    pub fn requires_ability<Ability>(
        self,
        ability: ApplicationAbilityRef<Schema, Ability, Scope>,
    ) -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope> {
        self.authorize(ApplicationQueryAuthorizationRequirement::for_ability(
            ability,
        ))
    }

    fn authorize(
        self,
        authorization: ApplicationQueryAuthorizationRequirement,
    ) -> ApplicationQueryDefinitionBuilder<Schema, Query, Parameters, QueryResult, Scope> {
        let Self { previous, lanes } = self;
        let ApplicationQueryLaneAuthoring {
            previous: basis,
            basis_support,
        } = previous;
        let disclosure = basis.previous;
        let dependency = disclosure.previous;
        let cardinality = dependency.previous;
        ApplicationQueryDefinitionBuilder::with_authorization(
            ApplicationQueryDefinitionParts {
                reference: cardinality.reference,
                root: cardinality.root,
                scope: cardinality.scope,
                result_shape: cardinality.result_shape,
                cardinality: dependency.cardinality,
                dependency_ceiling: disclosure.dependency_ceiling,
                disclosure: basis.disclosure,
                basis_support,
                lanes,
            },
            authorization,
        )
    }
}

pub(super) struct ApplicationQueryDefinitionParts<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Scope,
    Root,
> {
    pub(super) reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    pub(super) root: ApplicationEntityRef<Schema, Root>,
    pub(super) scope: ApplicationEntityRef<Schema, Scope>,
    pub(super) result_shape: ApplicationQueryResultShape,
    pub(super) cardinality: ApplicationQueryCardinality,
    pub(super) dependency_ceiling: ApplicationQueryDependencyCeiling,
    pub(super) disclosure: ApplicationQueryDisclosureContract,
    pub(super) basis_support: ApplicationQueryBasisSupport,
    pub(super) lanes: ApplicationQueryLaneEligibility,
}
