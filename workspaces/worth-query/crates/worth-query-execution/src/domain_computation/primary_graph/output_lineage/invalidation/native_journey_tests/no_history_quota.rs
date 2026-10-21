//! An absent position policy retains real, funded native source history.
use super::*;
use crate::domain_computation::execution_runtime::product_world::test_product_world_resources;
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::invalidation::commit_touches::CommitTouchEntities;
use crate::domain_computation::primary_graph::output_lineage::invalidation::commit_touches::CommitTouchInterest;

#[test]
fn no_position_policy_preserves_funded_history_and_original_reader_currentness() {
    let (_, clock, _, execution) = test_product_world_resources().into_parts();
    let resources = WorthQueryInvalidationResources::new();
    let configured = WorthQueryProductWorldResources::new(
        worth_runtime_world::facade::RuntimeWorldBudgets::without_policy_limits(),
        clock,
        resources,
        execution.policy,
    );
    let configured = match execution.authority {
        Some(authority) => configured.with_execution_authority(authority),
        None => configured,
    };
    let world = crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources(configured);
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let selected = world.selected_product(&custody);
        let entity = selected.resolve_entity(&phase, AccountStatus::reference(),
            "open".into(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary).unwrap().entity_id();
        let (_, product, _) = selected.into_parts();
        let coordinate = ProductCoordinate {
            occurrence: product.observation().lifecycle_incarnation(),
            generation: product.observation().reference_generation().get(),
        };
        let source = SemanticSource {
            runtime_authority: world.application.runtime.authority_identity().as_u64(),
            schema: world.application.installed_schema.binding_identity(),
            scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
            output_binding: OutputBindingIdentity::declared("StatusOutput"),
        };
        let row = RecordedSettlementIdentity::retain(&source, coordinate, 0);
        let graph = world.application.runtime.primary_graph().unwrap();
        let status_ref = AccountStatus::reference();
        let status = graph.layout().field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field()).unwrap().clone();
        let label_ref = AccountLabel::reference();
        let label = graph.layout().field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field()).unwrap().clone();
        let handle = graph.integration_handle();
        let owner = &handle.source_owner.invalidation_owner;
        let execution = custody.borrowed_request_for_test();
        handle.with_runtime_mut(&execution, |runtime| {
            let (original_handle, before) = snapshot(runtime);
            let original_fact = field_fact(&execution, runtime, &original_handle, entity, status.clone());
            register(owner, row.clone(), Arc::from([original_fact.clone()]), &before, OrdSet::new());

            let cell = owner.cell_for_read(&before).unwrap().unwrap();
            let historical = cell.read_image();
            for index in 0..300 {
                write_field(&execution, runtime, entity, label.clone(), &format!("unrelated-{index}"));
            }
            let (after_handle, after) = snapshot(runtime);
            let image = cell.read_image();
            assert!(image.payload().past.len() > 256);
            assert_eq!(image.payload().past.get(&historical.position()).unwrap().root_id, historical.root_id());

            assert!(matches!(currentness(owner, &after, &row), SourceSettlementCurrentness::Clean));
            assert_eq!(field_fact(&execution, runtime, &original_handle, entity, status.clone()), original_fact);
            let batch = owner.touched_commits_after(after.branch_id(), historical.position(), after.commit_id(), 300,
                CommitTouchInterest { entities: CommitTouchEntities::Slice(&[]), every_commit: true });
            assert_eq!(batch.commits.len(), 300);
            assert!(batch.caught_up_to_latest);
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
