use super::custom_rules::*;
use super::validation_engine_fixtures::*;
use structural_world::{candidate_lookups, StructuralWorld};

mod structural_world;

#[test]
fn engine_executes_custom_invariant_packets() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap())
        .build();

    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    match &results.results()[0].rule {
        InvariantReportedRule::Custom(identity) => {
            assert_eq!(identity.rule_id.as_str(), "test.custom.violation");
        }
        other => panic!("expected custom invariant result, got {other:?}"),
    }
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn leased_opaque_custom_rule_is_denied_before_evaluation() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(PanicDuringEvaluateRule).unwrap())
        .build();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let denial = InvariantEngine::new(&runtime)
        .execute(
            &worth_execution::ExecutionRequest::leased(
                &crate::tests::support::test_execution_lease(),
            ),
            request,
        )
        .expect_err("opaque custom execution must fail closed under a lease");
    assert!(matches!(
        denial,
        crate::transactions::data::TransactionCommitError::Execution {
            denial: crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                    crate::execution::RelationalExecutionDenialCause::UncheckedCustomKernel
                ),
                ..
            },
            ..
        }
    ));
}

#[test]
fn engine_executes_graph_composition_custom_invariant_packets() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(GraphCompositionViolatesCustomRule).unwrap(),
        )
        .build();

    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::GraphComposition,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].execution_point,
        crate::validation::data::InvariantExecutionPoint::GraphComposition
    );
    assert_eq!(
        results.metadata().max_cost(),
        crate::validation::data::InvariantCostClass::Touched
    );
    match &results.results()[0].rule {
        InvariantReportedRule::Custom(identity) => {
            assert_eq!(identity.rule_id.as_str(), "test.custom.graph-composition");
        }
        other => panic!("expected graph composition custom invariant result, got {other:?}"),
    }
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn engine_executes_custom_packets_against_real_structural_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let structural = || CustomInvariantRegistration::new(StructuralSurfaceRule).unwrap();
    let rejects = || CustomInvariantRegistration::new(RejectsPlannedRelationRule).unwrap();
    let shared = StructuralWorld::install(execution, [structural(), rejects()]);
    // Each rule installed alone materializes every input it asks for. Sharing
    // must answer exactly that demand, so the shared lookups equal their sum.
    let alone = [
        StructuralWorld::install(execution, [structural()]),
        StructuralWorld::install(execution, [rejects()]),
    ];
    let lookups_alone = |worlds: &[StructuralWorld; 2]| {
        worlds
            .iter()
            .map(|world| candidate_lookups(&world.evaluate().1))
            .sum::<usize>()
    };

    let (results, counters) = shared.evaluate();
    assert_eq!(results.results().len(), 2);
    assert!(matches!(
        results.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
    assert!(matches!(
        results.results()[1].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
    assert_eq!(counters.custom_invariant_preparation_count, 2);
    assert_eq!(counters.custom_invariant_execution_count, 2);
    assert_eq!(counters.custom_invariant_candidate_adjacency_gathers, 4);
    assert_eq!(counters.custom_invariant_candidate_adjacency_count_reads, 4);
    assert_eq!(
        counters.custom_invariant_candidate_adjacency_relation_ids,
        0
    );
    assert_eq!(counters.custom_invariant_candidate_entity_reads, 2);
    assert_eq!(candidate_lookups(&counters), lookups_alone(&alone));
    assert!(counters.custom_invariant_candidate_reuse_hits > 0);
    assert!(counters.custom_invariant_traversal_frontier_count >= 2);

    shared.commit_edge(execution);
    alone.iter().for_each(|world| world.commit_edge(execution));
    let (after_topology_change, changed_counters) = shared.evaluate();
    assert_eq!(after_topology_change.results().len(), 2);
    assert!(matches!(
        after_topology_change.results()[1].verdict,
        crate::validation::data::InvariantVerdict::Violation(_)
    ));
    assert_eq!(
        changed_counters.custom_invariant_candidate_relation_reads,
        1
    );
    assert_eq!(
        changed_counters.custom_invariant_candidate_adjacency_relation_ids,
        2
    );
    assert_eq!(candidate_lookups(&changed_counters), lookups_alone(&alone));
    assert!(changed_counters.custom_invariant_candidate_reuse_hits > 0);
}

#[test]
fn engine_captures_custom_prepare_panics_as_typed_failures() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(PanicDuringPrepareRule).unwrap())
        .build();
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::validation::data::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("expected captured prepare panic to produce a violation");
    };
    match &violation.fields {
        crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => {
            assert_eq!(
                identity.semantic_identity().rule_id.as_str(),
                "test.custom.panic-prepare"
            );
            assert_eq!(
                phase,
                &crate::validation::data::CustomInvariantFailurePhase::Preparation
            );
            assert_eq!(
                failure,
                &crate::validation::data::ResultCustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 0);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}

#[test]
fn engine_captures_custom_evaluate_panics_as_typed_failures() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(CustomInvariantRegistration::new(PanicDuringEvaluateRule).unwrap())
        .build();
    runtime.performance_access().reset_counters();

    let results = InvariantEngine::new(&runtime).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &runtime,
            InvariantObservation::committed(runtime.storage_access().current_edition()),
            runtime.current_version_id(),
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::validation::data::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("expected captured evaluate panic to produce a violation");
    };
    match &violation.fields {
        crate::validation::data::InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => {
            assert_eq!(
                identity.semantic_identity().rule_id.as_str(),
                "test.custom.panic-evaluate"
            );
            assert_eq!(
                phase,
                &crate::validation::data::CustomInvariantFailurePhase::Execution
            );
            assert_eq!(
                failure,
                &crate::validation::data::ResultCustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 1);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}
