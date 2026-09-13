mod cardinality;
mod deletion;
mod endpoints;

pub use cardinality::ApplicationRelationCardinality;
pub use deletion::ApplicationRelationDeletionPolicy;
pub use endpoints::{ApplicationRelationCrossContextPolicy, ApplicationRelationEndpoints};

use super::{ApplicationSchemaDeclarationDenial, ApplicationSchemaMember};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationIntegrity {
    pub endpoints: ApplicationRelationEndpoints,
    pub cardinality: ApplicationRelationCardinality,
    pub deletion: ApplicationRelationDeletionPolicy,
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
