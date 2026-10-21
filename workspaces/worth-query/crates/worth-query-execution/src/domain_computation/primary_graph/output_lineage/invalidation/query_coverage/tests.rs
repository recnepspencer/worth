//! Actual read backing, native release, touched-bucket cleanup and live marking.
use super::super::mark_state::MarkState;
use super::super::native_journey_tests::{field_fact, snapshot, write_field};
use super::super::settlement::PostingBindings;
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountLabel, AccountStatus,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

#[test]
fn released_query_backing_preserves_historical_truth_and_marks_only_live_readers() {
    let world = installed_authorization_world(true);
    let (historical, projection) = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host);
            let entity = selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "open".to_owned(),
                    &live_scope(),
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .entity_id();
            let graph = world.application.runtime.primary_graph().unwrap();
            let handle = graph.integration_handle();
            let owner = &handle.source_owner.invalidation_owner;
            let label_ref = AccountLabel::reference();
            let label = graph
                .layout()
                .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
                .unwrap()
                .clone();
            let status_ref = AccountStatus::reference();
            let status = graph
                .layout()
                .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
                .unwrap()
                .clone();
            handle.with_runtime_mut(&execution, |runtime| {
                write_field(&execution, runtime, entity, label.clone(), "prime");
                let (basis_handle, basis) = snapshot(runtime);
                let facts = [
                    field_fact(&execution, runtime, &basis_handle, entity, label.clone()),
                    field_fact(&execution, runtime, &basis_handle, entity, status),
                ];
                let capture = |facts: &[_]| {
                    owner
                        .retain_query_fact_coverage(
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
                let id = abandoned.slot.lock().unwrap().unwrap();
                let live_id = live.slot.lock().unwrap().unwrap();
                let projection = Arc::downgrade(&abandoned.projection);
                let key = FactPostingKey::FieldRevision {
                    entity,
                    aspect: label.aspect().aspect_key().clone(),
                    path: label.field_path().clone(),
                };
                let cell = owner.cell_for_read(&basis).unwrap().unwrap();
                let image = cell.read_image();
                let historical = Arc::clone(image.payload());
                let original_keys = query_binding(&historical.current, id)
                    .projection
                    .keys()
                    .len();
                assert_eq!(original_keys, 3);
                drop(image);
                drop(abandoned);
                assert!(!cell
                    .read_image()
                    .payload()
                    .current
                    .posting_bindings
                    .contains_key(&id));
                assert!(
                    projection.upgrade().is_some(),
                    "the original historical image retains its real backing"
                );

                // A historical state containing a dead native weak identity is the
                // same input a failed/racing final release leaves. Exercise only
                // the actual touched-bucket algorithm; never install it as authority.
                let mut deferred = (*historical.current).clone();
                assert_eq!(query_binding(&deferred, id).identity.strong_count(), 0);
                let mut counts = Default::default();
                marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                assert_eq!(
                    query_binding(&deferred, id).projection.keys().len(),
                    original_keys - 1
                );
                assert!(query_binding(&deferred, live_id).dirty.is_some());
                assert_eq!(counts.matched_fact_postings, 2);
                assert_eq!(counts.marked_fact_ordinals, 1);
                let before = counts.matched_fact_postings;
                marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                assert_eq!(
                    counts.matched_fact_postings - before,
                    1,
                    "a detached dead binding is not visited again"
                );
                let keys: Vec<_> = query_binding(&deferred, id)
                    .projection
                    .keys()
                    .keys()
                    .cloned()
                    .collect();
                for key in keys {
                    marking::mark_query_reads(&mut deferred, &key, &mut counts).unwrap();
                }
                assert!(!deferred.posting_bindings.contains_key(&id));
                assert!(deferred.postings.values().all(|ids| !ids.contains(&id)));
                drop(deferred);

                // Ordinary native publication still marks the live original read.
                write_field(&execution, runtime, entity, label, "changed");
                let (after_handle, after) = snapshot(runtime);
                assert!(query_binding(&cell.read_image().payload().current, live_id)
                    .dirty
                    .is_some());
                assert!(owner
                    .query_facts_clean_at(&live, &after, execution.allocation_policy())
                    .ok()
                    .flatten()
                    .is_none());
                assert_eq!(
                    query_binding(&historical.current, id)
                        .projection
                        .keys()
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

fn query_binding(state: &MarkState, id: u64) -> &QueryReadBinding {
    match state.posting_bindings.get(&id).unwrap().as_ref() {
        PostingBindings::QueryRead(read) => read,
        PostingBindings::Settlements(_) => panic!("query read is not a producer settlement"),
    }
}
