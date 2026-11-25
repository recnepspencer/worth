use crate::archive_decode_authority::ArchiveDecodeAuthority;
use std::marker::PhantomData;

use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};

mod authoring;
mod builder;
mod value_traits;

pub use authoring::{
    ApplicationQueryAuthorizationAuthoring, ApplicationQueryBasisAuthoring,
    ApplicationQueryCardinalityAuthoring, ApplicationQueryDisclosureAuthoring,
    ApplicationQueryLaneAuthoring, ApplicationQueryResultAuthoring, ApplicationQueryRootAuthoring,
    ApplicationQueryScopeAuthoring,
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

/// The right-hand side of a structured equality predicate: either a declared
/// runtime parameter or an authored scalar constant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationQueryPredicateExpected {
    Parameter(String),
    Value(AspectValue),
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationQueryPredicate {
    entity: String,
    aspect: String,
    field: String,
    expected: ApplicationQueryPredicateExpected,
    scalar_family: ScalarAspectType,
}

impl ApplicationQueryPredicate {
    pub fn from_untrusted_fields(
        _: ArchiveDecodeAuthority,
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
            expected: ApplicationQueryPredicateExpected::Parameter(parameter),
            scalar_family,
        }
    }

    pub fn from_untrusted_value(
        _: ArchiveDecodeAuthority,
        entity: String,
        aspect: String,
        field: String,
        value: AspectValue,
        scalar_family: ScalarAspectType,
    ) -> Self {
        Self {
            entity,
            aspect,
            field,
            expected: ApplicationQueryPredicateExpected::Value(value),
            scalar_family,
        }
    }

    pub fn field(&self) -> (&str, &str, &str) {
        (&self.entity, &self.aspect, &self.field)
    }

    pub fn parameter(&self) -> Option<&str> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(parameter) => Some(parameter),
            ApplicationQueryPredicateExpected::Value(_) => None,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match &self.expected {
            ApplicationQueryPredicateExpected::Parameter(_) => None,
            ApplicationQueryPredicateExpected::Value(value) => Some(value),
        }
    }

    pub const fn expected(&self) -> &ApplicationQueryPredicateExpected {
        &self.expected
    }

    pub const fn scalar_family(&self) -> ScalarAspectType {
        self.scalar_family
    }

    /// Declaration-owned local validity check for a structured literal expected
    /// value (canonical representation, exact non-null scalar family, portable
    /// string, and bounded byte width).
    pub fn literal_value_is_valid(value: &AspectValue, scalar_family: ScalarAspectType) -> bool {
        super::predicate_literal_is_valid(value, scalar_family)
    }
}

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
    pub(super) field_expectations: Vec<crate::application_schema::ApplicationFieldBindingRecipe>,
    pub(super) root_paths: Vec<ApplicationQueryRootPathMeaning>,
    pub(super) component_selection:
        Option<crate::application_query::ApplicationQueryComponentMeaning>,
    pub(super) orbit: Option<super::ApplicationQueryOrbitMeaning>,
    pub(super) managed_index: Option<super::ApplicationQueryManagedIndexMeaning>,
    pub(super) managed_index_recipe: Option<super::ApplicationManagedIndexSelectionRecipe>,
    pub(super) cardinality: ApplicationQueryCardinality,
    pub(super) predicates: Vec<ApplicationQueryPredicate>,
    pub(super) where_filter: Option<super::ApplicationQueryWhereFilterProgram>,
    pub(super) related_predicates: Vec<super::ApplicationQueryRelatedPredicateMeaning>,
    pub(super) ordering: Vec<ApplicationQueryOrderingTerm>,
    pub(super) continuation: Option<ApplicationQueryContinuationTarget>,
    pub(super) live_cause: Option<ApplicationQueryLiveCauseContract>,
    pub(super) live_whole_result: bool,
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

    pub fn component_selection(
        &self,
    ) -> Option<&crate::application_query::ApplicationQueryComponentMeaning> {
        self.component_selection.as_ref()
    }

    pub fn orbit(&self) -> Option<&crate::application_query::ApplicationQueryOrbitMeaning> {
        self.orbit.as_ref()
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

    pub fn where_filter(&self) -> Option<&super::ApplicationQueryWhereFilterProgram> {
        self.where_filter.as_ref()
    }

    pub fn related_predicates(&self) -> &[super::ApplicationQueryRelatedPredicateMeaning] {
        &self.related_predicates
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

    pub const fn is_live_whole_result(&self) -> bool {
        self.live_whole_result
    }

    pub const fn live_invalidation_mode(
        &self,
    ) -> Option<super::ApplicationQueryLiveInvalidationMode> {
        if self.live_whole_result {
            Some(super::ApplicationQueryLiveInvalidationMode::WholeResult)
        } else if self.live_cause.is_some() {
            Some(super::ApplicationQueryLiveInvalidationMode::CauseRouted)
        } else {
            None
        }
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

    pub(crate) fn take_field_expectations(
        &mut self,
    ) -> Vec<crate::application_schema::ApplicationFieldBindingRecipe> {
        std::mem::take(&mut self.field_expectations)
    }

    pub(crate) fn take_managed_index_recipe(
        &mut self,
    ) -> Option<super::ApplicationManagedIndexSelectionRecipe> {
        self.managed_index_recipe.take()
    }

    pub fn into_erased(self) -> super::ErasedApplicationQueryDefinition {
        super::ErasedApplicationQueryDefinition::from_typed(self)
    }
}
