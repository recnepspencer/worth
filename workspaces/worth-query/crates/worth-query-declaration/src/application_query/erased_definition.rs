mod parts;

pub use parts::WorthQueryPortableApplicationQueryParts;

use super::{
    ApplicationQueryAuthorizationRequirement, ApplicationQueryBasisSupport,
    ApplicationQueryCanonicalArtifact, ApplicationQueryCardinality,
    ApplicationQueryContinuationTarget, ApplicationQueryDefinition,
    ApplicationQueryDependencyCeiling, ApplicationQueryDependencyEquivalence,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryLiveCauseContract, ApplicationQueryOrderingTerm,
    ApplicationQueryParameterDefinition, ApplicationQueryPredicate, ApplicationQueryReference,
    ApplicationQueryResultShape, ApplicationQueryRootPathMeaning,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// Descriptive, authority-free application-query meaning retained by a
/// domain package.
///
/// Installation authority comes from resolving a typed reference against this
/// exact member of an installed application schema.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ErasedApplicationQueryDefinition {
    parts: WorthQueryPortableApplicationQueryParts,
    canonical: ApplicationQueryCanonicalArtifact,
}

impl ErasedApplicationQueryDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationQueryParts) -> Self {
        let canonical = super::canonical_basis::prepare_definition_basis(&parts);
        Self { parts, canonical }
    }

    pub const fn parts(&self) -> &WorthQueryPortableApplicationQueryParts {
        &self.parts
    }

    pub fn into_parts(self) -> WorthQueryPortableApplicationQueryParts {
        self.parts
    }

    pub fn name(&self) -> &str {
        &self.parts.name
    }

    pub fn query_type(&self) -> &str {
        self.parts.query_type.as_str()
    }

    pub fn query_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.parts.query_type.clone()
    }

    pub fn parameter_type(&self) -> &str {
        self.parts.parameter_type.as_str()
    }

    pub fn parameter_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.parts.parameter_type.clone()
    }

    pub fn result_type(&self) -> &str {
        self.parts.result_type.as_str()
    }

    pub fn result_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.parts.result_type.clone()
    }

    pub fn scope_type(&self) -> &str {
        self.parts.scope_type.as_str()
    }

    pub fn scope_identity(&self) -> WorthQueryPortableTypeIdentity {
        self.parts.scope_type.clone()
    }

    pub fn root_entity(&self) -> &str {
        &self.parts.root_entity
    }

    pub fn scope_entity(&self) -> &str {
        &self.parts.scope_entity
    }

    pub fn parameters(&self) -> &[ApplicationQueryParameterDefinition] {
        &self.parts.parameters
    }

    pub fn result_shape(&self) -> &ApplicationQueryResultShape {
        &self.parts.result_shape
    }

    pub fn root_paths(&self) -> &[ApplicationQueryRootPathMeaning] {
        &self.parts.root_paths
    }

    pub const fn cardinality(&self) -> ApplicationQueryCardinality {
        self.parts.cardinality
    }

    pub fn predicates(&self) -> &[ApplicationQueryPredicate] {
        &self.parts.predicates
    }

    pub fn ordering(&self) -> &[ApplicationQueryOrderingTerm] {
        &self.parts.ordering
    }

    pub const fn continuation(&self) -> Option<&ApplicationQueryContinuationTarget> {
        self.parts.continuation.as_ref()
    }

    pub const fn live_cause(&self) -> Option<&ApplicationQueryLiveCauseContract> {
        self.parts.live_cause.as_ref()
    }

    pub const fn dependency_ceiling(&self) -> ApplicationQueryDependencyCeiling {
        self.parts.dependency_ceiling
    }

    pub const fn dependency_equivalence(&self) -> ApplicationQueryDependencyEquivalence {
        self.parts.dependency_equivalence()
    }

    pub fn disclosure(&self) -> &ApplicationQueryDisclosureContract {
        &self.parts.disclosure
    }

    pub const fn authorization(&self) -> &ApplicationQueryAuthorizationRequirement {
        &self.parts.authorization
    }

    pub const fn basis_support(&self) -> ApplicationQueryBasisSupport {
        self.parts.basis_support
    }

    pub const fn lanes(&self) -> ApplicationQueryLaneEligibility {
        self.parts.lanes
    }

    pub fn canonical_basis(&self) -> &ApplicationQueryCanonicalArtifact {
        &self.canonical
    }

    pub fn matches_reference<Schema, Query, Parameters, QueryResult, Scope>(
        &self,
        reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> bool {
        self.parts.name == reference.name()
            && self.parts.query_type == reference.query_type()
            && self.parts.parameter_type == reference.parameter_type()
            && self.parts.result_type == reference.result_type()
            && self.parts.scope_type == reference.scope_type()
    }

    pub(super) fn from_typed<Schema, Query, Parameters, QueryResult, Scope>(
        definition: ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        Self::from_untrusted_parts(WorthQueryPortableApplicationQueryParts::from_typed(
            definition,
        ))
    }
}

#[cfg(test)]
#[path = "erased_definition/tests.rs"]
mod tests;
