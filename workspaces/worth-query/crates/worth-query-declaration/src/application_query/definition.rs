use std::marker::PhantomData;

use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::ScalarAspectType;

mod authoring;
mod builder;

pub use authoring::{
    ApplicationQueryAuthorizationAuthoring, ApplicationQueryBasisAuthoring,
    ApplicationQueryCardinalityAuthoring, ApplicationQueryDependencyAuthoring,
    ApplicationQueryDisclosureAuthoring, ApplicationQueryLaneAuthoring,
    ApplicationQueryResultAuthoring, ApplicationQueryRootAuthoring, ApplicationQueryScopeAuthoring,
};
pub use builder::ApplicationQueryDefinitionBuilder;

use super::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryContinuationTarget, ApplicationQueryDependencyEquivalence,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryLiveCauseContract, ApplicationQueryOrderingTerm,
    ApplicationQueryParameterDefinition, ApplicationQueryResultShape,
    ApplicationQueryRootPathMeaning,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryCardinality {
    OptionalOne,
    ExactlyOne,
    Many,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryPredicate {
    entity: String,
    aspect: String,
    field: String,
    parameter: String,
    scalar_family: ScalarAspectType,
}

impl ApplicationQueryPredicate {
    pub fn from_untrusted_fields(
        entity: String,
        aspect: String,
        field: String,
        parameter: String,
        scalar_family: ScalarAspectType,
    ) -> Self {
        Self {
            entity,
            aspect,
            field,
            parameter,
            scalar_family,
        }
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, &self.aspect, &self.field)
    }

    pub fn parameter(&self) -> &str {
        &self.parameter
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryDependencyCeiling {
    maximum_traversal_depth: usize,
    maximum_relation_count: usize,
    maximum_projected_field_count: usize,
}

impl ApplicationQueryDependencyCeiling {
    pub const fn bounded(
        maximum_traversal_depth: usize,
        maximum_relation_count: usize,
        maximum_projected_field_count: usize,
    ) -> Self {
        Self {
            maximum_traversal_depth,
            maximum_relation_count,
            maximum_projected_field_count,
        }
    }

    pub const fn maximum_traversal_depth(self) -> usize {
        self.maximum_traversal_depth
    }

    pub const fn maximum_relation_count(self) -> usize {
        self.maximum_relation_count
    }

    pub const fn maximum_projected_field_count(self) -> usize {
        self.maximum_projected_field_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope> {
    pub(super) name: &'static str,
    pub(super) query_type: WorthQueryPortableTypeIdentity,
    pub(super) parameter_type: WorthQueryPortableTypeIdentity,
    pub(super) result_type: WorthQueryPortableTypeIdentity,
    pub(super) scope_type: WorthQueryPortableTypeIdentity,
    pub(super) root_entity: &'static str,
    pub(super) scope_entity: &'static str,
    pub(super) parameters: Vec<ApplicationQueryParameterDefinition>,
    pub(super) result_shape: ApplicationQueryResultShape,
    pub(super) root_paths: Vec<ApplicationQueryRootPathMeaning>,
    pub(super) cardinality: ApplicationQueryCardinality,
    pub(super) predicates: Vec<ApplicationQueryPredicate>,
    pub(super) ordering: Vec<ApplicationQueryOrderingTerm>,
    pub(super) continuation: Option<ApplicationQueryContinuationTarget>,
    pub(super) live_cause: Option<ApplicationQueryLiveCauseContract>,
    pub(super) dependency_ceiling: ApplicationQueryDependencyCeiling,
    pub(super) disclosure: ApplicationQueryDisclosureContract,
    pub(super) authorization: ApplicationQueryAuthorizationRequirement,
    pub(super) basis_support: ApplicationQueryBasisSupport,
    pub(super) lanes: ApplicationQueryLaneEligibility,
    _marker: PhantomData<fn(Parameters) -> (Schema, Query, QueryResult, Scope)>,
}

impl<Schema, Query, Parameters, QueryResult, Scope>
    ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>
{
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn query_type(&self) -> &str {
        self.query_type.as_str()
    }

    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.query_type.clone()
    }

    pub const fn parameter_type(&self) -> &str {
        self.parameter_type.as_str()
    }

    pub fn parameter_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.parameter_type.clone()
    }

    pub const fn result_type(&self) -> &str {
        self.result_type.as_str()
    }

    pub fn result_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.result_type.clone()
    }

    pub const fn scope_type(&self) -> &str {
        self.scope_type.as_str()
    }

    pub fn scope_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.scope_type.clone()
    }

    pub const fn root_entity(&self) -> &'static str {
        self.root_entity
    }

    pub const fn scope_entity(&self) -> &'static str {
        self.scope_entity
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

    pub fn into_erased(self) -> super::ErasedApplicationQueryDefinition {
        super::ErasedApplicationQueryDefinition::from_typed(self)
    }
}
