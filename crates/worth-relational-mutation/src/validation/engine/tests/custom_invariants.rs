use super::custom_rules::*;
use super::validation_engine_fixtures::*;
use structural_world::{candidate_lookups, StructuralWorld};

pub mod structural_world;

#[test]
fn engine_executes_custom_invariant_packets() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(AlwaysViolatesCustomRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
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
        crate::contracts::validation::data::execution::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn leased_opaque_custom_rule_is_denied_before_evaluation() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(PanicDuringEvaluateRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed_branch(&partitions),
        version,
        None,
        None,
    );
    let denial = InvariantEngine::from_view(&view)
        .execute(
            &worth_execution::ExecutionRequest::leased(
                &worth_harness::facade::execution_fixture::test_execution_lease(),
            ),
            request,
        )
        .expect_err("opaque custom execution must fail closed under a lease");
    assert!(matches!(
        denial,
        crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::Execution {
            denial: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: crate::contracts::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
                    crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::UncheckedCustomKernel
                ),
                ..
            },
            ..
        }
    ));
}

#[test]
fn engine_executes_graph_composition_custom_invariant_packets() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(GraphCompositionViolatesCustomRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::GraphComposition,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    assert_eq!(
        results.results()[0].execution_point,
        crate::contracts::validation::data::execution::InvariantExecutionPoint::GraphComposition
    );
    assert_eq!(
        results.metadata().max_cost(),
        crate::contracts::validation::data::groups::InvariantCostClass::Touched
    );
    match &results.results()[0].rule {
        InvariantReportedRule::Custom(identity) => {
            assert_eq!(identity.rule_id.as_str(), "test.custom.graph-composition");
        }
        other => panic!("expected graph composition custom invariant result, got {other:?}"),
    }
    assert!(matches!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::Violation(_)
    ));
}

#[test]
fn engine_executes_custom_packets_against_real_structural_surfaces() {
    let structural = || CustomInvariantRegistration::new(StructuralSurfaceRule).unwrap();
    let rejects = || CustomInvariantRegistration::new(RejectsPlannedRelationRule).unwrap();
    let shared = StructuralWorld::install([structural(), rejects()]);
    // Each rule installed alone materializes every input it asks for. Sharing
    // must answer exactly that demand, so the shared lookups equal their sum,
    // and it must read fewer inputs physically: an input both rules ask for is
    // read once and reused, beyond what each rule already reuses alone.
    let alone = [
        StructuralWorld::install([structural()]),
        StructuralWorld::install([rejects()]),
    ];
    let lookups_and_reuse_alone = |worlds: &[StructuralWorld; 2]| {
        worlds.iter().fold((0, 0), |(lookups, reuse), world| {
            let counters = world.evaluate().1;
            (
                lookups + candidate_lookups(&counters),
                reuse + counters.custom_invariant_candidate_reuse_hits,
            )
        })
    };

    let (results, counters) = shared.evaluate();
    assert_eq!(results.results().len(), 2);
    assert!(matches!(
        results.results()[0].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::Pass
    ));
    assert!(matches!(
        results.results()[1].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::Violation(_)
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
    let (lookups, reuse) = lookups_and_reuse_alone(&alone);
    assert_eq!(candidate_lookups(&counters), lookups);
    assert!(counters.custom_invariant_candidate_reuse_hits > reuse);
    assert!(counters.custom_invariant_traversal_frontier_count >= 2);

    shared.install_committed_edge();
    alone
        .iter()
        .for_each(|world| world.install_committed_edge());
    let (after_topology_change, changed_counters) = shared.evaluate();
    assert_eq!(after_topology_change.results().len(), 2);
    assert!(matches!(
        after_topology_change.results()[1].verdict,
        crate::contracts::validation::data::execution::InvariantVerdict::Violation(_)
    ));
    assert_eq!(
        changed_counters.custom_invariant_candidate_relation_reads,
        1
    );
    assert_eq!(
        changed_counters.custom_invariant_candidate_adjacency_relation_ids,
        2
    );
    let (lookups, reuse) = lookups_and_reuse_alone(&alone);
    assert_eq!(candidate_lookups(&changed_counters), lookups);
    assert!(changed_counters.custom_invariant_candidate_reuse_hits > reuse);
}

#[test]
fn engine_captures_custom_prepare_panics_as_typed_failures() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(PanicDuringPrepareRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::contracts::validation::data::execution::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("expected captured prepare panic to produce a violation");
    };
    match &violation.fields {
        crate::contracts::validation::data::results::InvariantViolationFields::CustomInvariantFailure {
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
                &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase::Preparation
            );
            assert_eq!(
                failure,
                &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = instrumentation.complexity_snapshot();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 0);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}

#[test]
fn engine_captures_custom_evaluate_panics_as_typed_failures() {
    let instrumentation = crate::branch_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation::default();
    let version = crate::contracts::identity::data::primitives::VersionId(0);
    let mut config =
        crate::contracts::config::data::runtime_config::RelationalRuntimeConfig::default();
    config.schema.registry = RelationalSchemaRegistry::new();
    let view = crate::validation::engine::test_view::invariant_view_with_custom_rules(
        &instrumentation,
        config,
        version,
        vec![CustomInvariantRegistration::new(PanicDuringEvaluateRule).unwrap()],
    );
    let partitions = std::collections::BTreeMap::new();

    let results = InvariantEngine::from_view(&view).execute_fixture(
        InvariantExecutionRequest::from_profile_with_contract(
            InvariantRequestProfile::CommitBoundary,
            &view,
            InvariantObservation::committed_branch(&partitions),
            version,
            None,
            None,
        ),
    );

    assert_eq!(results.results().len(), 1);
    let crate::contracts::validation::data::execution::InvariantVerdict::Violation(violation) =
        &results.results()[0].verdict
    else {
        panic!("expected captured evaluate panic to produce a violation");
    };
    match &violation.fields {
        crate::contracts::validation::data::results::InvariantViolationFields::CustomInvariantFailure {
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
                &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailurePhase::Execution
            );
            assert_eq!(
                failure,
                &crate::contracts::validation::data::results::custom_invariant_failure::CustomInvariantFailureKind::Panic
            );
        }
        other => panic!("expected custom invariant failure fields, got {other:?}"),
    }
    assert_eq!(results.summary().custom_failure_count(), 1);
    assert_eq!(results.summary().custom_panic_count(), 1);
    let counters = instrumentation.complexity_snapshot();
    assert_eq!(counters.custom_invariant_preparation_count, 1);
    assert_eq!(counters.custom_invariant_execution_count, 1);
    assert_eq!(counters.custom_invariant_panic_count, 1);
}
