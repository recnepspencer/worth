pub(crate) use worth_query_worlds::facade::query::suite::graph_provider_execution::resource_admission::*;

#[test]
fn commit_provider_mismatch_denies_before_any_graph_contact() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_partial_effect_package(federated_touch_package::<
        RemoteA,
        RemoteB,
    >(true))
    .graph_participation(atomic_definition::<RemoteA>("remote-a"))
    .atomic_graph_participation_provider(
        RemoteA,
        SelectiveProvider::partial_effect(&log, None),
        SharedCommit,
    )
    .graph_participation(atomic_definition::<RemoteB>("remote-b"))
    .atomic_graph_participation_provider(
        RemoteB,
        SelectiveProvider::partial_effect(&log, None),
        SharedCommit,
    )
    .graph_commit_provider(
        SharedCommit,
        SelectiveProvider::commit_with_support(
            &log,
            vec!["remote-a", "remote-b"],
            mismatched_partial_effect_provider_resource_support(),
        ),
    )
    .workspace(request_execution, "graph-commit-resource-mismatch")
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        (),
        worth_query_worlds::facade::query::suite::installed_operation_fixture::partial_effect_execution_resource_request(),
        &workspace,
    ) else {
        panic!("commit-provider support mismatch must deny")
    };

    assert_resource_mismatch(&denial, "commit group");
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn graph_provider_mismatch_denies_before_any_graph_contact() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let log = Arc::new(Mutex::new(Vec::new()));
    let workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteA,
            SelectiveProvider::new_with_support(&log, mismatched_provider_resource_support()),
        )
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(RemoteB, SelectiveProvider::new(&log, None))
        .workspace(request_execution, "graph-provider-resource-mismatch")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, FederatedRead)
        .unwrap();

    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        (),
        worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &workspace,
    ) else {
        panic!("graph-provider support mismatch must deny")
    };

    assert_resource_mismatch(&denial, "graph role");
    assert!(log.lock().unwrap().is_empty());
}
