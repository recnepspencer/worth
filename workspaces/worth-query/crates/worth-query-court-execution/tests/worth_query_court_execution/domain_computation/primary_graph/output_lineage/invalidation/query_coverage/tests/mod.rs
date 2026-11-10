//! Actual read backing, native release, touched-bucket cleanup and live marking.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::mark_state::MarkState;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::field_fact;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::snapshot;
use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::write_field;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::settlement::posting_bindings::PostingBindings;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::fact_key::FactPostingKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::query_coverage::QueryReadBinding;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

#[test]
fn released_query_backing_preserves_historical_truth_and_marks_only_live_readers() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (historical, projection) = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let entity = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::at("AccountStatus"),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id_for_test();
            let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
            let handle = graph.integration_handle_for_test();
            let owner = (*handle.source_owner_for_test()).invalidation_owner_for_test();
            let label_ref = AccountLabel::at("AccountLabel");
            let label = graph
                .layout_for_test()
                .field_locator_for_test(label_ref.entity(), label_ref.aspect(), label_ref.field())
                .unwrap()
                .clone();
            let status_ref = AccountStatus::at("AccountStatus");
            let status = graph
                .layout_for_test()
                .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap()
                .clone();
            handle.with_runtime_mut_for_test(&execution, |runtime| {
                write_field(&execution, runtime, entity, label.clone(), "prime");
                let (basis_handle, basis) = snapshot(runtime);
                let facts = [
                    field_fact(&execution, runtime, &basis_handle, entity, label.clone()),
                    field_fact(&execution, runtime, &basis_handle, entity, status),
                ];
                let capture = |facts: &[_]| {
                    owner
                        .retain_query_fact_coverage_for_test(
                            basis.clone(),
                            facts,
                            execution.allocation_policy(),
                        )
                        .ok()
                        .flatten()
                        .unwrap()
                };
                let abandoned = capture(&facts);
                let live = capture(&facts[..1]);
                let id = (*abandoned.slot_for_test()).lock().unwrap().unwrap();
                let live_id = (*live.slot_for_test()).lock().unwrap().unwrap();
                let projection = Arc::downgrade(abandoned.projection_for_test());
                let key = FactPostingKey::FieldRevision {
                    entity,
                    aspect: label.aspect().aspect_key().clone(),
                    path: label.field_path().clone(),
                };
                let cell = owner.cell_for_read_for_test(&basis).unwrap().unwrap();
                let image = cell.read_image();
                let historical = Arc::clone(image.payload());
                let original_keys = (*query_binding(historical.current_for_test(), id)
                    .projection_for_test())
                    .keys_for_test()
                    .len();
                assert_eq!(original_keys, 3);
                drop(image);
                drop(abandoned);
                assert!(!(*(*cell
                    .read_image()
                    .payload()
                    .current_for_test())
                    .posting_bindings_for_test())
                    .contains_key(&id));
                assert!(
                    projection.upgrade().is_some(),
                    "the original historical image retains its real backing"
                );

                // A historical state containing a dead native weak identity is the
                // same input a failed/racing final release leaves. Exercise only
                // the actual touched-bucket algorithm; never install it as authority.
                let mut deferred = (*(*historical.current_for_test())).clone();
                assert_eq!((*query_binding(&deferred, id).identity_for_test()).strong_count(), 0);
                let mut counts = Default::default();
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::query_coverage::marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                assert_eq!(
                    (*query_binding(&deferred, id).projection_for_test()).keys_for_test().len(),
                    original_keys - 1
                );
                assert!((*query_binding(&deferred, live_id).dirty_for_test()).is_some());
                assert_eq!(counts.matched_fact_postings, 2);
                assert_eq!(counts.marked_fact_ordinals, 1);
                let before = counts.matched_fact_postings;
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::query_coverage::marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                assert_eq!(
                    counts.matched_fact_postings - before,
                    1,
                    "a detached dead binding is not visited again"
                );
                let keys: Vec<_> = (*query_binding(&deferred, id)
                    .projection_for_test())
                    .keys_for_test()
                    .keys()
                    .cloned()
                    .collect();
                for key in keys {
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::query_coverage::marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                }
                assert!(!(*deferred.posting_bindings_for_test()).contains_key(&id));
                assert!((*deferred.postings_for_test()).values().all(|ids| !ids.contains(&id)));
                drop(deferred);

                // Ordinary native publication still marks the live original read.
                write_field(&execution, runtime, entity, label, "changed");
                let (after_handle, after) = snapshot(runtime);
                assert!((*query_binding(cell.read_image().payload().current_for_test(), live_id)
                    .dirty_for_test())
                    .is_some());
                assert!(owner
                    .query_facts_clean_at_for_test(&live, &after, execution.allocation_policy())
                    .ok()
                    .flatten()
                    .is_none());
                assert_eq!(
                    (*query_binding(historical.current_for_test(), id)
                        .projection_for_test())
                        .keys_for_test()
                        .len(),
                    original_keys
                );
                drop(live);
                for handle in [basis_handle, after_handle] {
                    runtime.snapshots().release_snapshot(&handle).unwrap();
                }
                (historical, projection)
            })
        })
        .unwrap();
    drop(world);
    assert!(projection.upgrade().is_some());
    drop(historical);
    assert!(
        projection.upgrade().is_none(),
        "the final historical owner releases its actual backing"
    );
}

pub fn query_binding(state: &MarkState, id: u64) -> &QueryReadBinding {
    match (*state.posting_bindings_for_test())
        .get(&id)
        .unwrap()
        .as_ref()
    {
        PostingBindings::QueryRead(read) => read,
        PostingBindings::Settlements(_) => panic!("query read is not a producer settlement"),
    }
}
