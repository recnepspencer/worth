//! Joining a refreshed Stable row uses the source that admitted its current Ready.
use super::*;

#[test]
fn a_stale_handle_rejoins_the_stable_rows_current_source_without_reading_it_again() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, _) = limited_application(128 * 1_024 * 1_024, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, mut stale) = chain!(application, request);
    let mut driver = start_consumer!(application, request, "anchor-c");
    settled_in_one_advance!(driver, request, "the second consumer handle");
    for (step, y) in [2, 5].into_iter().enumerate() {
        change_root_input!(request, application, y, 0x612_8000_u64 + step as u64);
        settled_in_one_advance!(driver, request, "the driving consumer refresh");
    }
    let selected = request
        .query(PlanarRead {
            body_key: "anchor-c".to_owned(),
        })
        .execute()
        .unwrap();
    let root = selected.observed_sources()[0].root_entity_for_test();
    let head = request.retain_read().unwrap().selected_commit().clone();
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    let contacts = application.producer_contacts_at_root_on_this_thread_for_test(root);
    let result = settled_in_one_advance!(
        stale,
        request,
        "the stale handle rejoining a current Stable row"
    );
    assert_eq!(result.observation().selected_commit(), &head);
    assert_eq!(
        application.producer_contacts_at_root_on_this_thread_for_test(root),
        contacts
    );
    // The driving wave already rebuilt this dependent's source at this head.
    // Rejoining its current Ready reads that source zero additional times.
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&root).copied().unwrap_or(0) - before.get(&root).copied().unwrap_or(0),
        0
    );
    drop((a, b, driver, stale));
}
