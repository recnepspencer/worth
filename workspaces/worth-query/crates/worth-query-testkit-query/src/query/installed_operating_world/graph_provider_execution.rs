pub use std::sync::{Arc, Mutex};

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::domain;

pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::graph_read_material::graph_read_material;
pub use super::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    configured_runtime_for_understated_cost_package, federated_operation_contract_drift_package,
    federated_package, federated_touch_package, FederatedOperationContractDrift, FederatedRead,
    GeometryDomain, ReadFamily,
};

pub mod call_affinity;
pub mod contract_admission;
pub mod provider_fixture;
pub mod resource_admission;
pub use provider_fixture::*;

pub fn atomic_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
    definition(
        role,
        domain::WorthQueryGraphProjectionPosture::NativeProjection,
        domain::WorthQueryGraphMutationPosture::TouchAndEffect,
        domain::WorthQueryGraphCommitPosture::AtomicAuthorityRequired,
    )
}

pub fn read_definition<G>(
    role: &str,
    projection: domain::WorthQueryGraphProjectionPosture,
) -> domain::WorthQueryGraphParticipationDefinition<G> {
    definition(
        role,
        projection,
        domain::WorthQueryGraphMutationPosture::NotRequired,
        domain::WorthQueryGraphCommitPosture::ReadOnly,
    )
}

pub fn definition<G>(
    role: &str,
    projection: domain::WorthQueryGraphProjectionPosture,
    mutation: domain::WorthQueryGraphMutationPosture,
    commit: domain::WorthQueryGraphCommitPosture,
) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection,
            mutation,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit,
            failure: domain::WorthQueryGraphFailureTopology::BoundaryFailure,
        },
    )
}
