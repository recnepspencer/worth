use super::*;

#[test]
fn public_delivery_loads_through_the_registered_source_authority() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let envelope = field_change_envelope_for_source_role("model");
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime_with_delivery_source(
        exact_mapping(),
        envelope,
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };

    let TransitionOutcome::Success(counters) = runtime.deliver_installed_correspondence(
        &correspondence,
        &mut graph,
        crate::facade::RelationalCommittedPatchRequest::new(truth_commit(1)),
        execution,
    ) else {
        panic!("registered source publication should drive delivery")
    };
    assert_eq!(counters.source_load_attempts(), 1);
    assert_eq!(counters.source_envelopes_loaded(), 1);
    assert_eq!(counters.allocation_registry_lock_attempts(), 1);
    assert_eq!(counters.signal_capability_admissions(), 1);
    assert_eq!(counters.failed_deliveries(), 0);
    assert_eq!(counters.truth_targets_admitted(), 1);
    assert_eq!(counters.signal_seeds_emitted(), 1);
}

#[test]
fn public_delivery_rejects_harness_envelopes_from_a_registered_source() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime_with_delivery_source(
        exact_mapping(),
        field_change_envelope(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);

    assert!(matches!(
        runtime.deliver_installed_correspondence(
            &correspondence,
            &mut graph,
            crate::facade::RelationalCommittedPatchRequest::new(truth_commit(1)),
         execution),
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == crate::facade::BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch
                && denial.counters().source_load_attempts() == 1
                && denial.counters().source_envelopes_loaded() == 1
                && denial.counters().failed_deliveries() == 1
                && denial.counters().correspondence_lookups() == 0
    ));
    assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
}

#[test]
fn registered_source_cannot_substitute_another_commit_for_the_request() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime_with_delivery_source(
        exact_mapping(),
        field_change_envelope_for_source_role("model"),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);

    assert!(matches!(
        runtime.deliver_installed_correspondence(
            &correspondence,
            &mut graph,
            crate::facade::RelationalCommittedPatchRequest::new(truth_commit(2)),
         execution),
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == crate::facade::BridgeCorrespondenceDenialKind::CommittedPatchRequestMismatch
                && denial.counters().source_load_attempts() == 1
                && denial.counters().source_envelopes_loaded() == 1
                && denial.counters().failed_deliveries() == 1
                && denial.counters().correspondence_lookups() == 0
                && denial.counters().signal_capability_admissions() == 0
    ));
    assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
}

#[test]
fn foreign_relational_graph_role_denies_before_signal_mutation() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);

    assert!(matches!(
        runtime.deliver_installed_correspondence_envelope(
            &correspondence,
            &mut graph,
            &field_change_envelope_for_source_role("foreign-model"),
        ),
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == crate::facade::BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch
                && denial.counters().source_load_attempts() == 0
                && denial.counters().failed_deliveries() == 1
                && denial.counters().correspondence_lookups() == 0
    ));
    assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
}

#[test]
fn matching_role_cannot_hide_foreign_runtime_or_adapter_authority() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);

    for envelope in [
        field_change_envelope_for_source(100, "model", "relational-adapter:99"),
        field_change_envelope_for_source(99, "model", "foreign-adapter"),
    ] {
        assert!(matches!(
            runtime.deliver_installed_correspondence_envelope(
                &correspondence,
                &mut graph,
                &envelope,
            ),
            TransitionOutcome::Denied(denial)
                if denial.kind()
                    == crate::facade::BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch
                    && denial.counters().source_load_attempts() == 0
                    && denial.counters().failed_deliveries() == 1
                    && denial.counters().correspondence_lookups() == 0
        ));
        assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
    }
}

#[test]
fn copied_contract_identity_with_different_shape_conflicts_at_runtime_construction() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let base = dependency("query:one");
    let mut lookalike = base.clone();
    lookalike.contract = contract_with_extra_field();
    let error = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(exact_mapping())
        .register_semantic_correspondence(registration(base, vec![target(&graph, node)]))
        .register_semantic_correspondence(registration(lookalike, vec![target(&graph, node)]))
        .build()
        .expect_err("copied contract identity cannot hide shape drift");
    assert_eq!(
        error.kind(),
        BridgeBuildErrorKind::AmbiguousSemanticDependencyRegistration
    );
}

#[test]
fn unregistered_semantic_lookalike_is_denied_and_authoritative_indexes_rebuild_exactly() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Denied(denial) =
        runtime.install_semantic_correspondence(dependency("query:unregistered"), &graph)
    else {
        panic!("unregistered semantic dependency must deny");
    };
    assert_eq!(
        denial.kind(),
        crate::facade::BridgeCorrespondenceDenialKind::PortableDependencyNotInstalled
    );
    assert_eq!(denial.counters().semantic_dependency_lookups(), 1);
    assert_eq!(denial.counters().mapping_lookups(), 0);
    assert_eq!(denial.counters().signal_node_admissions(), 0);

    assert!(runtime
        .install_semantic_correspondence(dependency("query:one"), &graph)
        .is_success());
    let report = runtime
        .rebuild_correspondence_allocation_index()
        .expect("authoritative correspondence indexes rebuild");
    assert_eq!(report.authoritative_semantic_dependencies(), 1);
    assert_eq!(report.authoritative_allocation_records(), 1);
    assert_eq!(report.rebuilt_allocation_keys(), 1);
    assert!(report.exact_semantic_dependency_index_parity());
    assert!(report.exact_mapping_index_parity());
    assert!(report.exact_index_parity());
}

#[test]
fn every_portable_dependency_drift_denies_before_mapping_or_signal_work() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let base = dependency("query:one");
    let runtime = runtime(
        exact_mapping(),
        vec![registration(base.clone(), vec![target(&graph, node)])],
    );
    let mut cases = Vec::new();

    let mut basis = base.clone();
    basis.source_basis = Arc::from("source-basis:foreign");
    cases.push(basis);
    let mut runtime_authority = base.clone();
    runtime_authority.source_runtime_authority += 1;
    cases.push(runtime_authority);
    let mut installation_generation = base.clone();
    installation_generation.source_installation_generation += 1;
    cases.push(installation_generation);
    let mut graph_role = base.clone();
    graph_role.declared_graph_role = Arc::from("foreign-model");
    cases.push(graph_role);
    let mut graph_participation = base.clone();
    graph_participation.graph_participation_identity = Arc::from("foreign-participation");
    cases.push(graph_participation);
    let mut graph_adapter = base.clone();
    graph_adapter.graph_adapter_identity = Arc::from("foreign-adapter");
    cases.push(graph_adapter);
    let mut contract_revision = base.clone();
    contract_revision.contract = contract_at_revision(5);
    cases.push(contract_revision);
    let mut mask = base.clone();
    mask.projection_mask = AspectMask::whole_aspect();
    cases.push(mask);
    let mut binding = base.clone();
    binding.binding = AspectBinding::RelationSourceEndpoint;
    cases.push(binding);
    let mut locality = base.clone();
    locality.locality = BridgeSemanticLocality::WholeLogicalGraph;
    cases.push(locality);
    let mut changes = base;
    changes.relevant_changes = vec![AuthoritativeAspectChangeKind::FieldClear];
    cases.push(changes);

    for drifted in cases {
        let TransitionOutcome::Denied(denial) =
            runtime.install_semantic_correspondence(drifted, &graph)
        else {
            panic!("drifted portable dependency must not reach Bridge mapping admission");
        };
        assert_eq!(
            denial.kind(),
            crate::facade::BridgeCorrespondenceDenialKind::PortableDependencyNotInstalled
        );
        assert_eq!(denial.counters().semantic_dependency_lookups(), 1);
        assert_eq!(denial.counters().mapping_lookups(), 0);
        assert_eq!(denial.counters().signal_node_admissions(), 0);
        assert_eq!(denial.counters().targets_admitted(), 0);
    }
}
