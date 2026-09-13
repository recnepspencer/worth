use std::sync::{Arc, Mutex};

use worth_query::facade::domain;

use super::{
    configured_runtime_for_package, configured_runtime_for_understated_cost_package,
    federated_operation_contract_drift_package, federated_package, read_definition,
    FederatedOperationContractDrift, FederatedRead, GeometryDomain, ReadFamily, RemoteA, RemoteB,
    SelectiveProvider,
};

#[test]
fn graph_identity_and_cost_postures_are_real_binding_requirements() {
    for (name, package, understates_cost) in [
        (
            "identity",
            federated_operation_contract_drift_package::<RemoteA, RemoteB>(
                FederatedOperationContractDrift::PreserveLineage,
            ),
            false,
        ),
        (
            "cost",
            federated_operation_contract_drift_package::<RemoteA, RemoteB>(
                FederatedOperationContractDrift::UnderstatesExternalCost,
            ),
            true,
        ),
    ] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let runtime = if understates_cost {
            configured_runtime_for_understated_cost_package(package)
        } else {
            configured_runtime_for_package(package)
        };
        let workspace = runtime
            .graph_participation(read_definition::<RemoteA>(
                "remote-a",
                domain::WorthQueryGraphProjectionPosture::NativeProjection,
            ))
            .graph_participation_provider(RemoteA, SelectiveProvider::new(&log, None))
            .graph_participation(read_definition::<RemoteB>(
                "remote-b",
                domain::WorthQueryGraphProjectionPosture::NativeProjection,
            ))
            .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
            .workspace(format!("graph-contract-{name}"))
            .unwrap();
        assert_graph_contract_denial(&workspace, &log, name);
    }
}

#[test]
fn partial_commit_topology_requires_declared_operation_recovery() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(partial_commit_definition::<RemoteA>("remote-a"))
        .graph_participation_provider(RemoteA, SelectiveProvider::new(&log, None))
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace("graph-contract-partial-commit")
        .unwrap();
    assert_graph_contract_denial(&workspace, &log, "partial commit");
}

fn assert_graph_contract_denial(
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

fn partial_commit_definition<G>(role: &str) -> domain::WorthQueryGraphParticipationDefinition<G> {
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
