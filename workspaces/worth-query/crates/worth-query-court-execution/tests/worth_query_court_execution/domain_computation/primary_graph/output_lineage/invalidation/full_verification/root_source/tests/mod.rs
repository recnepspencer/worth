//! Completeness of retained upstream evidence against actual actor registration.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::verification::EvidenceView;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::full_verification::FullVerificationStop;
use im::OrdSet;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::invalidation::full_verification::root_source::tests;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn retained_source_must_include_every_registered_upstream_edge() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request_for_test()))
        .expect("installed host call opens");
    let entity = world
        .application
        .with_host_advancement(|phase| {
            selected.resolve_entity(
                &phase,
                AccountStatus::reference(),
                "open".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
        })
        .expect("installed host call opens")
        .unwrap()
        .entity_id_for_test();
    let (_, product, _) = selected.into_parts_for_test();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: (*world.application.runtime_for_test()).authority_identity().as_u64(),
        schema: (*world.application.installed_schema_for_test()).binding_identity(),
        scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity),
        output_binding,
    }
    };
    let upstream = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("UpstreamOutput")),
        coordinate,
        0,
    );
    let consumer = RecordedSettlementIdentity::retain_for_test(
        &source(OutputBindingIdentity::declared_for_test("ConsumerOutput")),
        coordinate,
        0,
    );
    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime_mut(request_execution, |runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let selected = Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap());
        for (identity, consumed_upstream) in [
            (Arc::clone(&upstream), OrdSet::new()),
            (Arc::clone(&consumer), OrdSet::unit(Arc::clone(&upstream))),
        ] {
            owner
                .register_settlement(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::SettlementRegistration {
                        computation: None,
                        work_membership: None,
                        identity,
                        facts: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::from([])),
                        output_facts: None,
                        read_basis: (*selected).clone(),
                        stale_at_read_basis: im::OrdSet::new(),
                        requirement: None,
                        upstream: consumed_upstream,
                    })
                .unwrap();
        }
        let retained = |identity, upstream| {
            ConsumedOutputEvidence::retained_for_test(identity, Arc::from([]), upstream, None, Arc::clone(&selected))
        };
        let omitted = retained(Arc::clone(&consumer), Vec::new());
        let complete = retained(Arc::clone(&consumer), vec![retained(upstream, Vec::new())]);

        let captured = owner.capture_full_verification(runtime, &snapshot, &selected).unwrap();
        assert_eq!(
            captured.require_complete_root_upstream(
                EvidenceView::from(&omitted),
                &consumer,
            ),
            Err(FullVerificationStop::OutputEvidenceUnavailable),
        );
        assert_eq!(
            captured.require_complete_root_upstream(
                EvidenceView::from(&complete),
                &consumer,
            ),
            Ok(()),
            "the control carries the actual registered upstream identity",
        );
        drop(captured);
        drop((omitted, complete, selected));
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
    });
}
