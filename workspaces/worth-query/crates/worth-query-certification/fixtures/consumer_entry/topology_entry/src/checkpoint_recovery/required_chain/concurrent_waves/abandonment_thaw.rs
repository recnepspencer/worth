//! Releasing the only deferred owner thaws unrelated work without a sweep.
use super::*;

fn case(cancel: bool, owner_name: &str, dirty_name: &str) {
    let application = support::install_program_with_seed::<application::Program>(
        None,
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard(),
        4096,
        128 * 1024 * 1024,
        1_000_000,
        diamond::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! root {
        ($key:expr) => {
            request
                .demand(PlanarOutputDemand::new($key))
                .start_in_program::<application::Program, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut owner = root!(owner_name);
    let mut dirty = root!(dirty_name);
    let mut other = root!("diamond-join");
    for demand in [&mut owner, &mut dirty, &mut other] {
        assert!(matches!(
            demand.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    let selected = request
        .query(PlanarRead {
            body_key: dirty_name.into(),
        })
        .execute()
        .unwrap();
    let dirty_root = selected.observed_sources()[0].root_entity_for_test();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: dirty_name.into(),
            replacement_y: length(if dirty_name == "diamond-left" { 5 } else { 54 }),
        })
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&0x9176_78ab_u64)
        .execute_performed::<application::Program, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    // A fresh demand on this occurrence produces the undelivered publication;
    // closing the prior cached answer's interest cannot deliver that new row.
    drop(owner);
    let mut owner = root!(owner_name);
    // Move the owner's own source as well, so it must publish a successor.
    let selected = request
        .query(PlanarRead {
            body_key: owner_name.into(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: owner_name.into(),
            replacement_y: length(if owner_name == "diamond-left" { 6 } else { 55 }),
        })
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&0x9176_78ac_u64)
        .execute_performed::<application::Program, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    application.delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        owner.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1)
    );
    let rows = application.queued_required_rows_for_root_for_test(dirty_root);
    assert!(!rows.is_empty());
    let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(true)
    );
    assert_eq!(
        application.queued_required_rows_for_root_for_test(dirty_root),
        rows
    );
    assert_eq!(
        primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test(),
        before
    );
    if cancel {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let stopped_scope =
            authentication::WorthQueryRequestScope::new(scope.deadline(), cancellation.token());
        let stopped_request = application.request(&principal, &stopped_scope);
        cancellation.cancel();
        let stopped = owner
            .advance(&stopped_request)
            .err()
            .expect("cancelled request is refused");
        assert!(matches!(stopped,
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(stop)
            if stop.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
                worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
                    worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))));
    }
    // The request is borrowed; abandonment closes its reusable demand interest.
    drop(owner);
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (0, 0)
    );
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(false)
    );
    let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(
        after.get(&dirty_root).copied().unwrap_or(0)
            - before.get(&dirty_root).copied().unwrap_or(0),
        1
    );
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let final_reads = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert_eq!(final_reads.get(&dirty_root), after.get(&dirty_root));
    assert!(application
        .queued_required_rows_for_root_for_test(dirty_root)
        .is_empty());
}

#[test]
fn abandoning_the_deferred_owner_thaws_the_next_other_caller() {
    let _guard = checkpoint_recovery_test_guard();
    for cancel in [false, true] {
        case(cancel, "diamond-left", "diamond-right");
        case(cancel, "diamond-right", "diamond-left");
    }
}
