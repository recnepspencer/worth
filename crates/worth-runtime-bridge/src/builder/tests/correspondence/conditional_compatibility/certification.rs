use super::*;

#[test]
fn installation_cost_is_independent_of_unrelated_admitted_dependencies() {
    let install = |name: &str, baseline: &[&str]| {
        let (mut owner, request) = installation_fixture_with_baseline(
            conditional_contract("query:one"),
            &["bridge-main"],
            BridgeConditionalProviderSet::new().compute(Compute(1)),
            baseline,
        );
        let lowering = owner.install(request).unwrap_or_else(|denial| {
            panic!(
                "{name} conditional installation denied: {}",
                denial.detail()
            )
        });
        (
            owner.active_semantic_dependency_count(),
            lowering.counters(),
        )
    };
    let (narrow_count, narrow) = install("narrow", &["query:first"]);
    let (wide_count, wide) = install(
        "wide",
        &["query:first", "query:second", "query:partition", "query:0"],
    );

    assert!(wide_count > narrow_count);
    assert_eq!(
        narrow.dependency_registry_existing_key_lookups,
        wide.dependency_registry_existing_key_lookups
    );
    assert_eq!(
        narrow.dependency_registry_batch_key_lookups,
        wide.dependency_registry_batch_key_lookups
    );
    assert_eq!(narrow.dependency_registry_commits, 1);
    assert_eq!(wide.dependency_registry_commits, 1);
    assert_eq!(narrow.unrelated_dependency_registry_scans, 0);
    assert_eq!(wide.unrelated_dependency_registry_scans, 0);
}

#[test]
fn conditional_execution_cost_ignores_unrelated_signal_nodes_and_dependencies() {
    let execute = |baseline: &[&str]| {
        let (mut owner, request) = installation_fixture_with_baseline(
            always_eligible_contract("query:one"),
            &["bridge-main"],
            BridgeConditionalProviderSet::new().compute(Compute(1)),
            baseline,
        );
        let lowering = owner.install(request).unwrap();
        let owner = owner.seal().unwrap();
        let signal_basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let decision = owner
            .execute(
                &signal_basis,
                crate::facade::BridgeConditionalExecutionRequest {
                    lowering: &lowering,
                    query_binding_identity: "query-binding-a",
                    query_capability_identity: 1,
                    snapshot_identity: "snapshot-a",
                    truth_branch_identity: None,
                    bridge_snapshot_identity: Some(
                        &crate::truth_identity_fixtures::truth_snapshot(1, 1),
                    ),
                    execution_identity: "execution-a",
                    attempt: 1,
                },
                &mut (),
            )
            .unwrap();
        let performed = decision
            .performed_signal_invalidation()
            .expect("initial conditional computation performs canonical Signal invalidation work");
        assert_eq!(
            performed.realized_counters().value(
                worth_signal::facade::adapters::InvalidationPerformedCounter::NodesEvaluated,
            ),
            1
        );
        decision.signal().counters()
    };
    let narrow = execute(&["query:first"]);
    let wide = execute(&["query:first", "query:second", "query:partition", "query:0"]);

    assert_eq!(narrow, wide);
    assert_eq!(narrow.request_admission_checks, 1);
    assert_eq!(narrow.contract_lookups, 1);
    assert_eq!(narrow.output_version_reads, 2);
    assert_eq!(narrow.application_contacts, 1);
    assert_eq!(narrow.semantic_classifications, 1);
}

#[test]
fn target_work_scales_with_the_owner_installed_correspondence_width() {
    let declaration = conditional_contract("query:one");
    let (_current_owner, current) = install_with_target_partitions(
        declaration.clone(),
        &["bridge-a", "bridge-b"],
        BridgeConditionalProviderSet::new().compute(Compute(1)),
    );
    let (_candidate_owner, candidate) = install_with_target_partitions(
        declaration,
        &["bridge-a", "bridge-b"],
        BridgeConditionalProviderSet::new().compute(Compute(1)),
    );

    let continuity = current.compare_semantic_continuity(&candidate).unwrap();
    let installed = current.counters();
    assert_eq!(installed.contract_admission_checks, 1);
    assert_eq!(installed.correspondence_registrations_inspected, 1);
    assert_eq!(installed.correspondence_targets_inspected, 2);
    assert_eq!(installed.signal_graph_checks, 1);
    assert_eq!(installed.signal_node_ownership_checks, 1);
    assert_eq!(installed.dependency_registry_compilations, 1);
    assert_eq!(installed.dependency_registry_existing_key_lookups, 2);
    assert_eq!(installed.dependency_registry_batch_key_lookups, 2);
    assert_eq!(installed.dependency_registry_commits, 1);
    assert_eq!(installed.unrelated_dependency_registry_scans, 0);
    assert_eq!(installed.semantic_observation_plan_compilations, 1);
    assert_eq!(installed.signal_node_admissions, 1);
    assert_eq!(installed.correspondence_batch_preparations, 1);
    assert_eq!(installed.signal_contract_lowerings, 1);
    assert_eq!(installed.correspondence_admissions, 1);
    assert_eq!(installed.signal_targets_joined, 2);
    assert_eq!(installed.signal_contract_installations, 1);
    assert_eq!(continuity.work().correspondences_inspected(), 1);
    assert_eq!(continuity.work().targets_inspected(), 2);
    assert_eq!(continuity.work().bridge_contract_comparisons(), 2);
}

#[test]
fn provider_denial_reports_exact_zero_downstream_installation_work() {
    let (mut owner, request) = installation_fixture(
        conditional_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new(),
    );
    let denial = match owner.install(request) {
        Ok(_) => panic!("missing compute provider must deny installation"),
        Err(denial) => denial,
    };
    let counters = denial.lowering_counters();
    assert_eq!(counters.contract_admission_checks, 1);
    assert_eq!(counters.provider_checks, 7);
    assert_eq!(counters.correspondence_registrations_inspected, 0);
    assert_eq!(counters.correspondence_targets_inspected, 0);
    assert_eq!(counters.signal_node_admissions, 0);
    assert_eq!(counters.correspondence_batch_preparations, 0);
    assert_eq!(counters.signal_contract_lowerings, 0);
    assert_eq!(counters.signal_contract_installations, 0);
}

#[test]
fn equivalent_reinstallation_preserves_semantics_but_not_execution_affinity() {
    let (_current_owner, current) = install(conditional_contract("query:one"), "bridge-main");
    let (_candidate_owner, candidate) = install(conditional_contract("query:one"), "bridge-main");

    assert_ne!(
        current.signal_graph_instance_id(),
        candidate.signal_graph_instance_id()
    );
    let continuity = current
        .compare_semantic_continuity(&candidate)
        .expect("owner-native meaning survives an honest reinstallation");
    assert!(Arc::ptr_eq(
        continuity.current_retention().lowering(),
        &current
    ));
    assert!(Arc::ptr_eq(
        continuity.candidate_liveness().retention().lowering(),
        &candidate
    ));
    assert_eq!(continuity.work().liveness_checks(), 1);
    assert_eq!(continuity.work().correspondences_inspected(), 1);
    assert_eq!(continuity.work().targets_inspected(), 1);
    assert_eq!(continuity.work().provider_roles_inspected(), 7);
    assert_eq!(continuity.work().signal_semantic_dimensions_inspected(), 8);
    assert_eq!(continuity.work().signal_affinity_dimensions_inspected(), 0);
    assert!(matches!(
        current
            .compare_execution_affinity(&candidate)
            .unwrap_err()
            .mismatch(),
        BridgeConditionalExecutionAffinityMismatch::SourceCorrespondenceAuthority { ordinal: 0 }
    ));
}

#[test]
fn exact_affinity_rechecks_revocable_owner_issued_liveness() {
    let (mut owner, lowering) = install(conditional_contract("query:one"), "bridge-main");
    let affinity = lowering
        .compare_execution_affinity(&lowering)
        .expect("the exact installed lowering retains affinity with itself");
    assert_eq!(affinity.work().liveness_checks(), 2);
    assert_eq!(affinity.work().correspondences_inspected(), 2);
    assert_eq!(affinity.work().targets_inspected(), 2);
    assert_eq!(affinity.work().provider_roles_inspected(), 14);
    assert_eq!(affinity.work().signal_semantic_dimensions_inspected(), 16);
    assert_eq!(affinity.work().signal_affinity_dimensions_inspected(), 7);
    assert_eq!(affinity.work().bridge_affinity_dimensions_inspected(), 1);
    let live = lowering
        .admit_live_conditional_lowering()
        .expect("installed lowering starts live");

    owner.revoke_conditional_liveness();
    assert!(!live.is_live());
    let retained = lowering.retain_conditional_lowering();
    assert!(Arc::ptr_eq(retained.lowering(), &lowering));
    assert!(matches!(
        lowering
            .compare_execution_affinity(&lowering)
            .unwrap_err()
            .mismatch(),
        BridgeConditionalExecutionAffinityMismatch::CurrentLoweringNotLive
    ));
}

#[test]
fn dropping_the_owner_revokes_liveness_but_retains_prior_meaning() {
    let lowering = {
        let (_owner, lowering) = install(conditional_contract("query:one"), "bridge-main");
        assert!(lowering
            .admit_live_conditional_lowering()
            .expect("installed lowering starts live")
            .is_live());
        lowering
    };

    assert!(lowering.admit_live_conditional_lowering().is_err());
    let retained = lowering.retain_conditional_lowering();
    assert!(Arc::ptr_eq(retained.lowering(), &lowering));
}

#[test]
fn target_partition_drift_denies_semantic_continuity() {
    let (_current_owner, current) = install(conditional_contract("query:one"), "bridge-main");
    let (_candidate_owner, candidate) = install(conditional_contract("query:one"), "bridge-other");

    assert!(matches!(
        current
            .compare_semantic_continuity(&candidate)
            .unwrap_err()
            .mismatch(),
        BridgeConditionalContinuityMismatch::TargetMeaning {
            ordinal: 0,
            target: 0
        }
    ));
}

#[test]
fn neutral_contract_drift_denies_before_correspondence_work() {
    let (_current_owner, current) = install(conditional_contract("query:one"), "bridge-main");
    let (_candidate_owner, candidate) =
        install(always_eligible_contract("query:one"), "bridge-main");

    let denial = current.compare_semantic_continuity(&candidate).unwrap_err();
    assert_eq!(
        denial.mismatch(),
        &BridgeConditionalContinuityMismatch::ConditionalContract
    );
    assert_eq!(denial.work().bridge_contract_comparisons(), 1);

    let affinity_denial = current.compare_execution_affinity(&candidate).unwrap_err();
    assert!(matches!(
        affinity_denial.mismatch(),
        BridgeConditionalExecutionAffinityMismatch::Continuity(
            BridgeConditionalContinuityMismatch::ConditionalContract
        )
    ));
    assert_eq!(affinity_denial.work().liveness_checks(), 2);
    assert_eq!(
        affinity_denial.work().bridge_contract_comparisons(),
        denial.work().bridge_contract_comparisons()
    );
}

#[test]
fn reconstitution_preserves_owner_definition_and_rebuilds_derived_correspondences() {
    let mut graph = SignalGraph::new();
    let baseline_node = graph.node().build();
    let conditional_node_id = graph.node().build();
    let worth_proof::TransitionOutcome::Success(baseline_capability) =
        graph.admit_installed_node(baseline_node)
    else {
        panic!("baseline node admits");
    };
    let worth_proof::TransitionOutcome::Success(conditional_capability) =
        graph.admit_installed_node(conditional_node_id)
    else {
        panic!("conditional node admits");
    };
    let declared_target = |node| {
        BridgeSignalAspectTargetDeclaration::allocate(
            BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
            PartitionToken::new("bridge-main"),
            node,
        )
    };
    let worth_proof::TransitionOutcome::Success(baseline_exact_capability) =
        graph.admit_installed_aspect(baseline_node, Aspect::new(0))
    else {
        panic!("baseline exact aspect admits");
    };
    let baseline_exact_target = BridgeSignalAspectTargetDeclaration::exact(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
        PartitionToken::new("bridge-main"),
        baseline_capability,
        baseline_exact_capability,
    )
    .expect("baseline exact target shares graph authority");
    let worth_proof::TransitionOutcome::Success(baseline_allocate_capability) =
        graph.admit_installed_node(baseline_node)
    else {
        panic!("baseline allocation node admits");
    };
    let baseline_allocate_target = declared_target(baseline_allocate_capability);
    let baseline_targets = vec![baseline_exact_target, baseline_allocate_target];
    let baseline_dependency = freshly_installed_dependency("query:first");
    let baseline = registration(baseline_dependency.clone(), baseline_targets.clone());
    let conditional = registration(
        freshly_installed_dependency("query:one"),
        vec![declared_target(conditional_capability)],
    );
    let runtime = runtime(exact_mapping(), vec![baseline]);
    let baseline_aspects = {
        let mut binding = runtime
            .bind_signal_graph(&mut graph)
            .expect("the baseline Bridge runtime must bind its Signal graph");
        let worth_proof::TransitionOutcome::Success(installed) =
            binding.install_semantic_correspondence(baseline_dependency.clone())
        else {
            panic!("the baseline semantic correspondence must install")
        };
        installed
            .targets()
            .map(|target| target.aspect())
            .collect::<Vec<_>>()
    };
    assert_eq!(baseline_aspects, vec![Aspect::new(0), Aspect::new(1)]);
    let mut owner = BridgeConditionalRuntimeBuilder::new(
        runtime,
        Box::new(graph),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .expect("Bridge owns the runtime with builder authority");
    let incumbent = owner
        .install(BridgeConditionalInstallationRequest {
            contract: conditional_contract("query:one"),
            location: BridgeConditionalLocation::operation("query:one"),
            registrations: vec![conditional],
            providers: BridgeConditionalProviderSet::new().compute(Compute(1)),
        })
        .expect("conditional authority extends the baseline registry");
    assert_eq!(owner.baseline_semantic_dependency_count(), 1);
    assert_eq!(owner.active_semantic_dependency_count(), 2);
    let mut owner = owner.seal().expect("conditional owner seals once");
    owner.destroy_reconstitutable_indexes_for_test();
    let graph = owner.owned_signal_graph_instance_id();
    let basis = owner.admitted_signal_basis().clone();
    let candidate = owner
        .prepare_conditional_reconstitution(&basis)
        .expect("same owner readmits derived services");
    let readmitted = candidate.readmit_lowering(&incumbent).unwrap();
    assert_eq!(readmitted.signal_node(), incumbent.signal_node());
    assert!(incumbent.compare_semantic_continuity(&readmitted).is_ok());
    let report = owner
        .activate_conditional_reconstitution(candidate)
        .unwrap();
    assert_eq!(report.signal_graph_instance_id(), graph);
    assert_eq!(owner.owned_signal_graph_instance_id(), graph);
    assert_eq!(report.signal().service_readmission_count(), 1);
    assert_eq!(report.readmitted_lowering_count(), 1);
    assert!(report
        .correspondence()
        .exact_semantic_dependency_index_parity());
    assert!(report.correspondence().exact_mapping_index_parity());
    assert!(report.correspondence().exact_index_parity());
    assert!(incumbent.compare_execution_affinity(&readmitted).is_err());
}
