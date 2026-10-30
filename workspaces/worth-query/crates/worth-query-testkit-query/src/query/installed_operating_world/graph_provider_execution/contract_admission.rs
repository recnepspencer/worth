pub use std::sync::{Arc, Mutex};

pub use worth_query::facade::domain;

pub use super::{
    configured_runtime_for_package, configured_runtime_for_understated_cost_package,
    federated_operation_contract_drift_package, federated_package, read_definition,
    FederatedOperationContractDrift, FederatedRead, GeometryDomain, ReadFamily, RemoteA, RemoteB,
    SelectiveProvider,
};

pub fn assert_graph_contract_denial(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
    log: &Arc<Mutex<Vec<&'static str>>>,
    name: &str,
) {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let denial = match workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, FederatedRead)
    {
        Ok(_) => panic!("{name} contract drift unexpectedly bound"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::GraphAuthorityInsufficient
    );
    assert_eq!(denial.counters().graph_participation_lookups, 1);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert!(log.lock().unwrap().is_empty());
}

pub fn partial_commit_definition<G>(
    role: &str,
) -> domain::WorthQueryGraphParticipationDefinition<G> {
    domain::WorthQueryGraphParticipationDefinition::new(
        role,
        domain::WorthQueryGraphParticipationContract {
            observation: domain::WorthQueryGraphObservationPosture::Snapshot,
            projection: domain::WorthQueryGraphProjectionPosture::NativeProjection,
            mutation: domain::WorthQueryGraphMutationPosture::NotRequired,
            identity: domain::WorthQueryGraphIdentityPosture::Opaque,
            locality: domain::WorthQueryGraphLocalityPosture::ExternalBoundary,
            budget: domain::WorthQueryGraphBudgetPosture::ExternalBoundary,
            commit: domain::WorthQueryGraphCommitPosture::CompensationRequired,
            failure: domain::WorthQueryGraphFailureTopology::PartialCommitPossible,
        },
    )
}
