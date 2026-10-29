pub(crate) use worth_query_worlds::facade::query::suite::graph_participation::*;

#[test]
fn adapter_declaration_order_converges_without_provider_contact() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let direct_contacts = Arc::new(AtomicUsize::new(0));
    let direct = configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&direct_contacts))
        .graph_participation(definition::<OtherGraph>("other"))
        .graph_participation_provider(OtherGraph, CountingProvider::new(&direct_contacts))
        .workspace(request_execution, "graph-order-direct")
        .unwrap();
    let reversed_contacts = Arc::new(AtomicUsize::new(0));
    let reversed = configured_runtime()
        .graph_participation_provider(OtherGraph, CountingProvider::new(&reversed_contacts))
        .graph_participation(definition::<OtherGraph>("other"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&reversed_contacts))
        .graph_participation(definition::<RemoteGraph>("remote"))
        .workspace(request_execution, "graph-order-reversed")
        .unwrap();
    let direct_adapter = direct.graph_participation(RemoteGraph).unwrap();
    let reversed_adapter = reversed.graph_participation(RemoteGraph).unwrap();
    assert_eq!(direct_adapter.role(), reversed_adapter.role());
    assert_eq!(direct_adapter.contract(), reversed_adapter.contract());
    assert_eq!(direct_contacts.load(Ordering::Relaxed), 0);
    assert_eq!(reversed_contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn definition_provider_sets_must_close_exactly() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = Arc::new(AtomicUsize::new(0));
    assert!(configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .workspace(request_execution, "graph-missing-provider")
        .is_err());
    assert!(configured_runtime()
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&contacts))
        .workspace(request_execution, "graph-extra-provider")
        .is_err());
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn atomic_graphs_require_one_exact_referenced_commit_provider() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = Arc::new(AtomicUsize::new(0));
    assert!(configured_runtime()
        .graph_participation(atomic_definition::<AtomicGraph>("atomic"))
        .atomic_graph_participation_provider(
            AtomicGraph,
            CountingProvider::new(&contacts),
            AtomicCommit,
        )
        .workspace(request_execution, "graph-missing-commit-provider")
        .is_err());
    assert!(configured_runtime()
        .graph_commit_provider(AtomicCommit, CountingProvider::new(&contacts))
        .workspace(request_execution, "graph-extra-commit-provider")
        .is_err());
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}

#[test]
fn matching_role_on_another_marker_has_no_authority() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let contacts = Arc::new(AtomicUsize::new(0));
    let world = configured_runtime()
        .graph_participation(definition::<RemoteGraph>("remote"))
        .graph_participation_provider(RemoteGraph, CountingProvider::new(&contacts))
        .workspace(request_execution, "graph-marker-lookalike")
        .unwrap();
    let denial = match world.graph_participation(RemoteGraphLookalike) {
        Ok(_) => panic!("matching role on a foreign marker must not resolve"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryGraphParticipationLookupDenialKind::NotInstalled
    );
    assert_eq!(denial.counters().indexed_lookups, 1);
    assert_eq!(denial.counters().provider_contacts, 0);
    assert_eq!(contacts.load(Ordering::Relaxed), 0);
}
