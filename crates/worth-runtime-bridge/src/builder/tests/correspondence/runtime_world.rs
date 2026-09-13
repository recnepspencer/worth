use super::*;

#[test]
fn runtime_world_admission_port_counts_direct_currentness_lookups_and_shares_its_ledger() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let dependency = dependency("query:one");
    let runtime = runtime(
        exact_mapping(),
        vec![registration(dependency.clone(), vec![target(&graph, node)])],
    );
    let correspondence = {
        let mut binding = runtime
            .bind_signal_graph(&mut graph)
            .expect("the fixture graph binds to the issuing Bridge runtime");
        let TransitionOutcome::Success(correspondence) =
            binding.install_semantic_correspondence(dependency)
        else {
            panic!("installed correspondence");
        };
        correspondence
    };
    let port = runtime.runtime_world_correspondence_port();
    let admitted = port
        .admit_installed_basis(&correspondence)
        .expect("the issuing Bridge runtime admits its current basis");
    assert_eq!(port.inspection_counters().binding_index_lookups(), 1);

    let clone = port.clone();
    clone
        .compare_current_exact(&admitted)
        .expect("a cloned port shares the currentness ledger");
    assert_eq!(port.inspection_counters().binding_index_lookups(), 2);

    let fresh = runtime.runtime_world_correspondence_port();
    fresh
        .compare_current_exact(&admitted)
        .expect("a fresh port still performs one direct lookup");
    assert_eq!(fresh.inspection_counters().binding_index_lookups(), 1);
    assert_eq!(port.inspection_counters().binding_index_lookups(), 2);

    let foreign = runtime.fork_managed_request_lane();
    let foreign_port = foreign.runtime_world_correspondence_port();
    assert!(matches!(
        foreign_port.compare_current_exact(&admitted),
        Err(crate::facade::RuntimeWorldCorrespondenceAdmissionDenial::ForeignBridgeRuntime { .. })
    ));
    assert_eq!(
        foreign_port.inspection_counters().binding_index_lookups(),
        0,
        "foreign runtime denial precedes the direct binding lookup"
    );
}

#[test]
fn runtime_world_admission_preserves_exact_basis_and_admission_identity() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let dependency = dependency("query:one");
    let runtime = runtime(
        exact_mapping(),
        vec![registration(dependency.clone(), vec![target(&graph, node)])],
    );
    let correspondence = {
        let mut binding = runtime
            .bind_signal_graph(&mut graph)
            .expect("the fixture graph binds to the issuing Bridge runtime");
        let TransitionOutcome::Success(correspondence) =
            binding.install_semantic_correspondence(dependency)
        else {
            panic!("installed correspondence");
        };
        correspondence
    };

    let port = runtime.runtime_world_correspondence_port();
    let admitted = port
        .admit_installed_basis(&correspondence)
        .expect("the issuing Bridge runtime admits the installed basis");
    let repeated = port
        .admit_installed_basis(&correspondence)
        .expect("re-admission of the same installed basis remains exact");

    assert_eq!(admitted, repeated);
    assert_eq!(admitted.basis(), correspondence.basis());
    assert_eq!(
        admitted.admission_identity(),
        correspondence.admission_identity()
    );
    assert_eq!(
        admitted.source_installation_generation(),
        correspondence.basis().source_installation_generation()
    );
    assert_eq!(
        admitted.signal_graph_instance_id(),
        correspondence.basis().signal_graph_instance_id
    );
    port.compare_current_exact(&admitted)
        .expect("the admitted basis is current");
    assert_eq!(port.inspection_counters().binding_index_lookups(), 3);
}

#[test]
fn runtime_world_admission_denies_generation_drift_before_component_effects() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let stale_dependency = dependency("query:one");
    let mut current_dependency = stale_dependency.clone();
    current_dependency.source_installation_generation += 1;
    let runtime = runtime(
        exact_mapping(),
        vec![
            registration(stale_dependency.clone(), vec![target(&graph, node)]),
            registration(current_dependency, vec![target(&graph, node)]),
        ],
    );
    let correspondence = {
        let mut binding = runtime
            .bind_signal_graph(&mut graph)
            .expect("the fixture graph binds to the issuing Bridge runtime");
        let TransitionOutcome::Success(correspondence) =
            binding.install_semantic_correspondence(stale_dependency)
        else {
            panic!("installed stale correspondence");
        };
        correspondence
    };
    let aspect = correspondence
        .targets()
        .next()
        .expect("installed target")
        .aspect();
    let version_before_admission = graph.node_aspect_version(node).unwrap().get(aspect);

    let port = runtime.runtime_world_correspondence_port();
    assert!(matches!(
        port.admit_installed_basis(&correspondence),
        Err(
            crate::facade::RuntimeWorldCorrespondenceAdmissionDenial::InstalledGenerationDrift {
                expected_generation: 2,
                actual_generation: 1,
            }
        )
    ));
    assert_eq!(port.inspection_counters().binding_index_lookups(), 1);
    assert_eq!(
        graph.node_aspect_version(node).unwrap().get(aspect),
        version_before_admission,
        "denial does not mutate the Signal graph"
    );
}

#[test]
fn runtime_world_admission_currentness_is_constant_in_registration_population() {
    for population in [1_usize, 64, 512, 4096] {
        let mut graph = SignalGraph::new();
        let node = graph.node().build();
        let mut registrations = Vec::with_capacity(population);
        let mut selected_dependency = None;
        for slot in 0..population {
            let candidate = semantic_dependencies::freshly_installed_dependency(&format!(
                "query:runtime-world-scale:{population}:{slot}"
            ));
            if slot + 1 == population {
                selected_dependency = Some(candidate.clone());
            }
            registrations.push(registration(candidate, vec![target(&graph, node)]));
        }

        let runtime = runtime(exact_mapping(), registrations);
        let dependency = selected_dependency.expect("the scale fixture installs one candidate");
        let correspondence = {
            let mut binding = runtime
                .bind_signal_graph(&mut graph)
                .expect("the fixture graph binds to the issuing Bridge runtime");
            let TransitionOutcome::Success(correspondence) =
                binding.install_semantic_correspondence(dependency)
            else {
                panic!("installed scale correspondence");
            };
            correspondence
        };

        let port = runtime.runtime_world_correspondence_port();
        let admitted = port
            .admit_installed_basis(&correspondence)
            .expect("the current scale correspondence is admitted");
        assert_eq!(port.inspection_counters().binding_index_lookups(), 1);
        port.compare_current_exact(&admitted)
            .expect("the scale correspondence remains current");
        assert_eq!(port.inspection_counters().binding_index_lookups(), 2);
    }
}
