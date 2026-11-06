//! An absent position policy retains real, funded native source history.
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::retained_headroom::*;
use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::resources::test_product_world_resources;
use worth_query_execution::facade::integration::WorthQueryProductWorldResources;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::commit_touches::CommitTouchEntities;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::commit_touches::CommitTouchInterest;

#[test]
fn no_position_policy_preserves_funded_history_and_original_reader_currentness() {
    let (_, clock, _, execution) = test_product_world_resources(
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
    .into_parts_for_test();
    // This history court funds all 301 publications; finite work has its own court.
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        execution.policy_for_test().posture(),
        execution.policy_for_test().determinism(),
        worth_foundational::ExecutionBudget::new(
            execution.policy_for_test().max_workers(),
            execution.policy_for_test().charged_memory_bytes(),
            None,
        ),
    );
    let resources = WorthQueryInvalidationResources::new();
    let configured = WorthQueryProductWorldResources::new(
        worth_runtime_world::facade::RuntimeWorldBudgets::without_policy_limits(),
        clock,
        resources,
        policy,
    );
    let configured = match execution.authority_into_for_test() {
        Some(authority) => configured.with_execution_authority(authority),
        None => configured,
    };
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world_with_product_resources(configured);
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request_for_test();
        let selected = world.selected_product(&custody);
        let entity = selected.resolve_entity(&phase, AccountStatus::reference(),
            "open".into(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id_for_test();
        let (_, product, _) = selected.into_parts_for_test();
        let coordinate = ProductCoordinate::new_for_test(product.observation_for_test().lifecycle_incarnation(), product.observation_for_test().reference_generation().get());
        let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity), OutputBindingIdentity::declared_for_test("StatusOutput"));
        let row = RecordedSettlementIdentity::retain_for_test(&source, coordinate, 0);
        let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
        let status_ref = AccountStatus::reference();
        let status = graph.layout_for_test().field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field()).unwrap().clone();
        let label_ref = AccountLabel::reference();
        let label = graph.layout_for_test().field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field()).unwrap().clone();
        let handle = graph.integration_handle_for_test();
        let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
        let execution = custody.borrowed_request_for_test();
        handle.with_runtime_for_test(&execution, |runtime| {
            let (original_handle, before) = snapshot(runtime);
            let original_fact = field_fact(&execution, runtime, &original_handle, entity, status.clone());
            register(owner, row.clone(), Arc::from([original_fact.clone()]), &before, OrdSet::new());

            let cell = owner.cell_for_read_for_test(&before).unwrap().unwrap();
            let historical = cell.read_image();
            for index in 0..300 {
                write_field(&execution, runtime, entity, label.clone(), &format!("unrelated-{index}"));
            }
            let (after_handle, after) = snapshot(runtime);
            let image = cell.read_image();
            assert!((*image.payload().past_for_test()).len() > 256);
            assert_eq!((*(*image.payload().past_for_test()).get(&historical.position()).unwrap().root_id_for_test()), historical.root_id());

            assert!(matches!(currentness(owner, &after, &row), SourceSettlementCurrentness::Clean));
            assert_eq!(field_fact(&execution, runtime, &original_handle, entity, status.clone()), original_fact);
            let batch = owner.touched_commits_after_for_test(after.branch_id(), historical.position(), after.commit_id(), 300,
                CommitTouchInterest::new_for_test(CommitTouchEntities::Slice(&[]), true));
            assert_eq!((*batch.commits_for_test()).len(), 300);
            assert!((*batch.caught_up_to_latest_for_test()));
            write_field(&execution, runtime, entity, status, "closed");
            let (changed_handle, changed) = snapshot(runtime);
            assert!(matches!(currentness(owner, &changed, &row), SourceSettlementCurrentness::Dirty(_)));
            for snapshot in [original_handle, after_handle, changed_handle] {
                runtime.snapshots().release_snapshot(&snapshot).unwrap();
            }
        });
    }).unwrap();
    drop(world);
}
