mod cardinality;
mod deletion;
mod endpoints;
mod law_meaning;
mod uniqueness;
pub(crate) use law_meaning::ApplicationRelationLawMeaning;
pub use uniqueness::ApplicationRelationUniquenessScope;

pub use cardinality::ApplicationRelationCardinality;
pub use deletion::ApplicationRelationDeletionPolicy;
pub use endpoints::{ApplicationRelationCrossContextPolicy, ApplicationRelationEndpoints};

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationIntegrity {
    pub endpoints: ApplicationRelationEndpoints,
    pub cardinality: ApplicationRelationCardinality,
    pub deletion: ApplicationRelationDeletionPolicy,
    /// Every source-to-target path must be free of cycles at the commit boundary.
    pub acyclic: bool,
    /// Each surviving directed edge must have its reverse on this same relation.
    pub symmetric: bool,
    /// Physical edge multiplicity is bounded by the declared semantic scope.
    pub unique: Option<ApplicationRelationUniquenessScope>,
}

impl ApplicationRelationIntegrity {
    pub const fn new(
        endpoints: ApplicationRelationEndpoints,
        cardinality: ApplicationRelationCardinality,
        deletion: ApplicationRelationDeletionPolicy,
    ) -> Self {
        Self {
            endpoints,
            cardinality,
            deletion,
            acyclic: false,
            symmetric: false,
            unique: None,
        }
    }

    pub const fn same_context_unbounded_retain_dangling() -> Self {
        Self::new(
            ApplicationRelationEndpoints::same_context(true),
            ApplicationRelationCardinality::unbounded(),
            ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        )
    }

    pub const fn same_context_no_self_edges_unbounded_retain_dangling() -> Self {
        Self::new(
            ApplicationRelationEndpoints::same_context(false),
            ApplicationRelationCardinality::unbounded(),
            ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        )
    }

    pub const fn cross_context_policy(self) -> ApplicationRelationCrossContextPolicy {
        self.endpoints.cross_context_policy
    }

    /// Refuse a proposed commit whose edges close a directed cycle.
    pub const fn acyclic(mut self) -> Self {
        self.acyclic = true;
        self
    }

    /// Require the reverse of every surviving source-to-target edge.
    pub const fn symmetric(mut self) -> Self {
        self.symmetric = true;
        self
    }

    /// Refuse physical duplicate edges within this declared scope.
    pub const fn unique(mut self, scope: ApplicationRelationUniquenessScope) -> Self {
        self.unique = Some(scope);
        self
    }
}

pub(super) fn validate_relation_integrity(
    members: &[ApplicationSchemaMember],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    if members.iter().any(|member| {
        matches!(
            member,
            ApplicationSchemaMember::Relation { integrity, .. }
                if !integrity.cardinality.is_valid()
        )
    }) {
        Err(ApplicationSchemaDeclarationDenial::InvalidRelationIntegrity)
    } else {
        Ok(())
    }
}
