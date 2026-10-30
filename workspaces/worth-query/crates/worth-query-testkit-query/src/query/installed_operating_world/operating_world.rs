pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::sync::Arc;
pub use worth_query::facade::domain;

pub use super::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    federated_package, federated_touch_package, required_domain_runtime, workspace, FederatedRead,
    GeometryDomain, ReadFamily, ReadVertex,
};

pub mod provider_fixture;
pub use provider_fixture::*;

pub fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::TouchAndEffect,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::AtomicAuthorityRequired,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
