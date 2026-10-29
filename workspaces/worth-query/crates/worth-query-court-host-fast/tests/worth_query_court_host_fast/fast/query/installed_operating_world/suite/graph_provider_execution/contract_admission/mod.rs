pub(crate) use worth_query_worlds::facade::query::suite::graph_provider_execution::contract_admission::*;

#[test]
fn graph_identity_and_cost_postures_are_real_binding_requirements() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
            .workspace(request_execution, format!("graph-contract-{name}"))
            .unwrap();
        assert_graph_contract_denial(&workspace, &log, name);
    }
}

#[test]
fn partial_commit_topology_requires_declared_operation_recovery() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(partial_commit_definition::<RemoteA>("remote-a"))
        .graph_participation_provider(RemoteA, SelectiveProvider::new(&log, None))
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace(request_execution, "graph-contract-partial-commit")
        .unwrap();
    assert_graph_contract_denial(&workspace, &log, "partial commit");
}
