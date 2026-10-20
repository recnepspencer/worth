//! Failed final release cannot hand an old read footprint to a one-field writer.
use super::super::{
    logical_marking::WorthQueryNativeMarkingPrecision,
    mark_state::MarkState,
    native_journey_tests::{field_fact, snapshot, write_field},
    settlement::PostingBindings,
};
use super::*;
use crate::domain_computation::primary_graph::{
    tests::fixture::{installed_authorization_world, live_scope, AccountLabel, AccountStatus},
    WorthQueryPrincipalResolutionMode,
};

#[test]
fn failed_final_query_release_reclaims_only_touched_buckets_and_keeps_grants() {
    let world = installed_authorization_world(true);
    let (historical, projection_grant, identity_grant, resources) = world.application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host.borrowed_request_for_test();
            let selected = world.selected_product(&host.phase_for_application(&world.application).test_host_request());
            let entity = selected.resolve_entity(
                &host.phase_for_application(&world.application), AccountStatus::reference(),
                "open".to_owned(), &live_scope(), WorthQueryPrincipalResolutionMode::Ordinary,
            ).unwrap().entity_id();
            let graph = world.application.runtime.primary_graph().unwrap();
            let handle = graph.integration_handle();
            let owner = &handle.source_owner.invalidation_owner;
            // Layout locators and revisions below come from the installed schema
            // and genuine native reads, not caller-described dirty keys.
            let label_ref = AccountLabel::reference();
            let label = graph.layout().field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field()).unwrap().clone();
            let status_ref = AccountStatus::reference();
            let status = graph.layout().field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field()).unwrap().clone();
            handle.with_runtime_mut(&execution, |runtime| {
                write_field(&execution, runtime, entity, label.clone(), "prime");
                let (basis_handle, basis) = snapshot(runtime);
                let facts = [
                    field_fact(&execution, runtime, &basis_handle, entity, label.clone()),
                    field_fact(&execution, runtime, &basis_handle, entity, status),
                ];
                let capture = |facts: &[_]| owner.retain_query_fact_coverage(
                    basis.clone(), facts, execution.allocation_policy(), &mut owner.edit_admission(),
                ).ok().flatten().expect("genuine original native facts admit coverage");
                let abandoned = capture(&facts);
                let live = capture(&facts[..1]);
                let id = abandoned.slot.lock().unwrap().unwrap();
                let live_id = live.slot.lock().unwrap().unwrap();
                let original_keys = abandoned.projection.keys.len();
                assert_eq!(original_keys, 3, "two field addresses and their shared lifecycle");
                let projection_grant = Arc::downgrade(&abandoned.projection._capacity);
                let identity_grant = Arc::downgrade(&abandoned._capacity);
                let resources = owner.resources.clone();
                let maximum = resources.installation().maximum_retained_bytes.expect("this court installs a finite retained ceiling");
                let held = resources.reserve_retained_capacity(maximum - resources.retained_capacity_bytes()).unwrap();
                assert_eq!(resources.retained_capacity_bytes(), maximum);
                drop(abandoned); // The REAL final Drop cannot fund its native CAS.
                let cell = owner.cell_for_read(&basis, &mut owner.edit_admission()).unwrap().unwrap();
                let image = cell.read_image();
                let historical = Arc::clone(image.payload());
                let dead = query_binding(&historical.current, id);
                assert_eq!(dead.identity.strong_count(), 0);
                assert_eq!(dead.projection.keys().len(), original_keys, "refused release installed nothing");
                drop(image);
                drop(held);
                assert!(projection_grant.upgrade().is_some());
                assert!(identity_grant.upgrade().is_some());

                let observation = crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
                super::super::marking_observation::native_marking_on_this_thread_for_test();
                write_field(&execution, runtime, entity, label.clone(), "changed");
                let (after_handle, after) = snapshot(runtime);
                let after_image = cell.read_image();
                let state = &after_image.payload().current;
                let dead = query_binding(state, id);
                assert_eq!(dead.projection.keys().len(), original_keys - 1,
                    "the one-field writer detached ONE bucket, not the old projection");
                assert!(matches!(dead.projection, QueryReadBacking::Released { .. }));
                assert!(query_binding(state, live_id).dirty.is_some(), "cleanup cannot skip a live reader");
                assert!(owner.query_facts_clean_at(&live, &after, execution.allocation_policy(), &mut owner.edit_admission()).ok().flatten().is_none());
                assert_eq!(query_binding(&historical.current, id).projection.keys().len(), original_keys,
                    "historical roots keep their exact original backing");
                drop(after_image);
                let first = measured_work(owner, &after);
                write_field(&execution, runtime, entity, label, "again");
                let (repeat_handle, repeat) = snapshot(runtime);
                assert_eq!(query_binding(&cell.read_image().payload().current, id).projection.keys().len(), original_keys - 1,
                    "the detached bucket cannot be charged or removed again");
                let repeated = measured_work(owner, &repeat);
                assert!(first >= repeated);
                println!("failed-query-release original_keys={original_keys} remaining_keys={} first_work={first} repeated_work={repeated}", original_keys - 1);
                drop(observation);

                // Exercise the same native cleanup on the remaining real keys.
                // The whole-key list is test oracle work, never a writer input.
                let image = cell.read_image();
                let keys: Vec<_> = query_binding(&image.payload().current, id).projection.keys().keys().cloned().collect();
                let mut admission = owner.edit_admission();
                let before = admission.index_checkpoint();
                admission.bytes(index_capacity::arc_bytes::<MarkState>().unwrap()).unwrap();
                let mut next = (*image.payload().current).clone();
                for key in keys {
                    marking::mark_query_reads(&mut next, &key, &mut admission, &mut Default::default()).unwrap();
                }
                assert!(!next.posting_bindings.contains_key(&id));
                assert!(next.postings.values().all(|ids| !ids.contains(&id)));
                owner.install_live_state(cell, image, (next, before), &mut admission).unwrap();
                assert!(projection_grant.upgrade().is_some(), "historical original roots still hold their grant");
                drop(live);
                for handle in [basis_handle, after_handle, repeat_handle] {
                    runtime.snapshots().release_snapshot(&handle).unwrap();
                }
                (historical, projection_grant, identity_grant, resources)
            })
        }).expect("fixture owns its installed native request");
    drop(world);
    assert!(projection_grant.upgrade().is_some());
    assert!(identity_grant.upgrade().is_some());
    let before = resources.retained_capacity_bytes();
    drop(historical);
    assert!(
        projection_grant.upgrade().is_none(),
        "final original backing releases its direct grant"
    );
    assert!(
        identity_grant.upgrade().is_none(),
        "last historical weak identity releases its grant"
    );
    assert!(resources.retained_capacity_bytes() < before);
}

fn query_binding(state: &MarkState, id: u64) -> &QueryReadBinding {
    match state.posting_bindings.get(&id).unwrap().as_ref() {
        PostingBindings::QueryRead(read) => read,
        PostingBindings::Settlements(_) => panic!("query read is not a producer settlement"),
    }
}
fn measured_work(owner: &SourceInvalidationOwner, basis: &PositionedRelationalSnapshot) -> u64 {
    let report = owner
        .native_marking_report(basis, &mut owner.edit_admission())
        .unwrap()
        .unwrap();
    assert!(
        matches!(report.precision, WorthQueryNativeMarkingPrecision::Exact(_)),
        "one-field cleanup must retain exact delivery"
    );
    super::super::marking_observation::native_marking_on_this_thread_for_test()
        .into_iter()
        .find(|entry| entry.commit == report.commit && entry.precision == report.precision)
        .expect("the actual committed native preparation was observed")
        .charged_work
        .expect("exact native work was measured")
}
