pub(crate) use worth_query_worlds::facade::query::suite::graph_provider_execution::call_affinity::*;

#[test]
fn retained_call_cannot_replace_the_current_sealed_step() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let retained = Arc::new(Mutex::new(None));
    let mut workspace = configured_runtime_for_package(federated_package::<RemoteA, RemoteB>())
        .graph_participation(read_definition::<RemoteA>(
            "remote-a",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteA,
            RetainingProvider {
                retained: Arc::clone(&retained),
                projection_label: "remote-a-projection",
            },
        )
        .graph_participation(read_definition::<RemoteB>(
            "remote-b",
            domain::WorthQueryGraphProjectionPosture::NativeProjection,
        ))
        .graph_participation_provider(
            RemoteB,
            RetainingProvider {
                retained: Arc::clone(&retained),
                projection_label: "remote-b-projection",
            },
        )
        .workspace(request_execution, "graph-provider-retained-call")
        .unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();

    for _ in 0..2 {
        workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed, FederatedRead)
            .unwrap()
            .admit_execution_resources(
                (),
                worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
                &workspace,
            )
            .unwrap()
            .execute(&mut workspace)
            .unwrap();
    }

    let retained = retained
        .lock()
        .unwrap()
        .clone()
        .expect("provider retained the first descriptive call");
    assert_eq!(retained.graph_role(), "remote-a");
}
