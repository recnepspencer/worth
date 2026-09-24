use super::super::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryContinuationTarget, ApplicationQueryDefinition,
    ApplicationQueryDependencyCeiling, ApplicationQueryDependencyEquivalence,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryLiveCauseContract, ApplicationQueryOrderingTerm,
    ApplicationQueryParameterDefinition, ApplicationQueryPredicate, ApplicationQueryResultShape,
    ApplicationQueryRootPathMeaning,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryPortableApplicationQueryParts {
    pub name: String,
    pub query_type: WorthQueryPortableTypeIdentity,
    pub parameter_type: WorthQueryPortableTypeIdentity,
    pub result_type: WorthQueryPortableTypeIdentity,
    pub scope_type: WorthQueryPortableTypeIdentity,
    pub root_entity: String,
    pub scope_entity: String,
    pub parameters: Vec<ApplicationQueryParameterDefinition>,
    pub result_shape: ApplicationQueryResultShape,
    pub root_paths: Vec<ApplicationQueryRootPathMeaning>,
    pub cardinality: ApplicationQueryCardinality,
    pub predicates: Vec<ApplicationQueryPredicate>,
    pub ordering: Vec<ApplicationQueryOrderingTerm>,
    pub continuation: Option<ApplicationQueryContinuationTarget>,
    pub live_cause: Option<ApplicationQueryLiveCauseContract>,
    pub dependency_ceiling: ApplicationQueryDependencyCeiling,
    pub disclosure: ApplicationQueryDisclosureContract,
    pub authorization: ApplicationQueryAuthorizationRequirement,
    pub basis_support: ApplicationQueryBasisSupport,
    pub lanes: ApplicationQueryLaneEligibility,
}

impl WorthQueryPortableApplicationQueryParts {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn parameter_type(&self) -> &str {
        self.parameter_type.as_str()
    }

    pub fn result_type(&self) -> &str {
        self.result_type.as_str()
    }

    pub fn scope_type(&self) -> &str {
        self.scope_type.as_str()
    }

    pub fn root_entity(&self) -> &str {
        &self.root_entity
    }

    pub fn scope_entity(&self) -> &str {
        &self.scope_entity
    }

    pub fn parameters(&self) -> &[ApplicationQueryParameterDefinition] {
        &self.parameters
    }

    pub fn result_shape(&self) -> &ApplicationQueryResultShape {
        &self.result_shape
    }

    pub fn root_paths(&self) -> &[ApplicationQueryRootPathMeaning] {
        &self.root_paths
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        self.cardinality
    }

    pub fn predicates(&self) -> &[ApplicationQueryPredicate] {
        &self.predicates
    }

    pub fn ordering(&self) -> &[ApplicationQueryOrderingTerm] {
        &self.ordering
    }

    pub const fn continuation(&self) -> Option<&ApplicationQueryContinuationTarget> {
        self.continuation.as_ref()
    }

    pub const fn live_cause(&self) -> Option<&ApplicationQueryLiveCauseContract> {
        self.live_cause.as_ref()
    }

    pub const fn dependency_ceiling(&self) -> ApplicationQueryDependencyCeiling {
        self.dependency_ceiling
    }

    pub const fn dependency_equivalence(&self) -> ApplicationQueryDependencyEquivalence {
        ApplicationQueryDependencyEquivalence::CompleteNativeSourceRevisions
    }

    pub fn disclosure(&self) -> &ApplicationQueryDisclosureContract {
        &self.disclosure
    }

    pub const fn authorization(&self) -> &ApplicationQueryAuthorizationRequirement {
        &self.authorization
    }

    pub const fn basis_support(&self) -> ApplicationQueryBasisSupport {
        self.basis_support
    }

    pub const fn lanes(&self) -> ApplicationQueryLaneEligibility {
        self.lanes
    }

    pub(super) fn from_typed<Schema, Query, Parameters, QueryResult, Scope>(
        definition: ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        Self {
            name: definition.name.to_owned(),
            query_type: definition.query_type,
            parameter_type: definition.parameter_type,
            result_type: definition.result_type,
            scope_type: definition.scope_type,
            root_entity: definition.root_entity.to_owned(),
            scope_entity: definition.scope_entity.to_owned(),
            parameters: definition.parameters,
            result_shape: definition.result_shape,
            root_paths: definition.root_paths,
            cardinality: definition.cardinality,
            predicates: definition.predicates,
            ordering: definition.ordering,
            continuation: definition.continuation,
            live_cause: definition.live_cause,
            dependency_ceiling: definition.dependency_ceiling,
            disclosure: definition.disclosure,
            authorization: definition.authorization,
            basis_support: definition.basis_support,
            lanes: definition.lanes,
        }
    }

    pub(crate) fn project_typed<Schema, Query, Parameters, QueryResult, Scope>(
        definition: &ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        Self {
            name: definition.name.to_owned(),
            query_type: definition.query_type.clone(),
            parameter_type: definition.parameter_type.clone(),
            result_type: definition.result_type.clone(),
            scope_type: definition.scope_type.clone(),
            root_entity: definition.root_entity.to_owned(),
            scope_entity: definition.scope_entity.to_owned(),
            parameters: definition.parameters.clone(),
            result_shape: definition.result_shape.clone(),
            root_paths: definition.root_paths.clone(),
            cardinality: definition.cardinality,
            predicates: definition.predicates.clone(),
            ordering: definition.ordering.clone(),
            continuation: definition.continuation.clone(),
            live_cause: definition.live_cause.clone(),
            dependency_ceiling: definition.dependency_ceiling,
            disclosure: definition.disclosure.clone(),
            authorization: definition.authorization.clone(),
            basis_support: definition.basis_support,
            lanes: definition.lanes,
        }
    }
}
