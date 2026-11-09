//! Query definitions carry metadata, independent of marker trait derivations.
use super::*;
impl<Schema, Query, Parameters, QueryResult, Scope> Clone
    for ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            query_type: self.query_type.clone(),
            parameter_type: self.parameter_type.clone(),
            result_type: self.result_type.clone(),
            scope_type: self.scope_type.clone(),
            root_entity: self.root_entity,
            scope_entity: self.scope_entity,
            parameters: self.parameters.clone(),
            result_shape: self.result_shape.clone(),
            field_expectations: self.field_expectations.clone(),
            root_paths: self.root_paths.clone(),
            component_selection: self.component_selection.clone(),
            orbit: self.orbit.clone(),
            managed_index: self.managed_index.clone(),
            managed_index_recipe: self.managed_index_recipe.clone(),
            cardinality: self.cardinality,
            predicates: self.predicates.clone(),
            ordering: self.ordering.clone(),
            continuation: self.continuation.clone(),
            live_cause: self.live_cause.clone(),
            disclosure: self.disclosure.clone(),
            authorization: self.authorization.clone(),
            basis_support: self.basis_support,
            lanes: self.lanes,
            _marker: PhantomData,
        }
    }
}
impl<Schema, Query, Parameters, QueryResult, Scope> std::fmt::Debug
    for ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApplicationQueryDefinition")
            .field("name", &self.name)
            .field("query_type", &self.query_type)
            .field("parameter_type", &self.parameter_type)
            .field("result_type", &self.result_type)
            .field("scope_type", &self.scope_type)
            .field("root_entity", &self.root_entity)
            .field("scope_entity", &self.scope_entity)
            .field("parameters", &self.parameters)
            .field("result_shape", &self.result_shape)
            .field("field_expectations", &self.field_expectations)
            .field("root_paths", &self.root_paths)
            .field("component_selection", &self.component_selection)
            .field("orbit", &self.orbit)
            .field("managed_index", &self.managed_index)
            .field("managed_index_recipe", &self.managed_index_recipe)
            .field("cardinality", &self.cardinality)
            .field("predicates", &self.predicates)
            .field("ordering", &self.ordering)
            .field("continuation", &self.continuation)
            .field("live_cause", &self.live_cause)
            .field("disclosure", &self.disclosure)
            .field("authorization", &self.authorization)
            .field("basis_support", &self.basis_support)
            .field("lanes", &self.lanes)
            .finish()
    }
}
impl<Schema, Query, Parameters, QueryResult, Scope> PartialEq
    for ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.query_type == other.query_type
            && self.parameter_type == other.parameter_type
            && self.result_type == other.result_type
            && self.scope_type == other.scope_type
            && self.root_entity == other.root_entity
            && self.scope_entity == other.scope_entity
            && self.parameters == other.parameters
            && self.result_shape == other.result_shape
            && self.field_expectations == other.field_expectations
            && self.root_paths == other.root_paths
            && self.component_selection == other.component_selection
            && self.orbit == other.orbit
            && self.managed_index == other.managed_index
            && self.managed_index_recipe == other.managed_index_recipe
            && self.cardinality == other.cardinality
            && self.predicates == other.predicates
            && self.ordering == other.ordering
            && self.continuation == other.continuation
            && self.live_cause == other.live_cause
            && self.disclosure == other.disclosure
            && self.authorization == other.authorization
            && self.basis_support == other.basis_support
            && self.lanes == other.lanes
    }
}
impl<Schema, Query, Parameters, QueryResult, Scope> Eq
    for ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
}
