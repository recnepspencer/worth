//! Two published occurrences can be delivered only through their own custody.
use super::*;
use std::sync::Arc;

#[test]
fn two_owners_deliver_only_their_own_awaited_row() {
    let _guard = checkpoint_recovery_test_guard();
    let application = Arc::new(support::install_program_with_seed::<application::Program>(
        None,
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard(),
        4096,
        128 * 1024 * 1024,
        Some(1_000_000),
        diamond::seed,
    ));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    macro_rules! root {
        ($name:expr) => {
            request
                .demand(PlanarOutputDemand::new($name))
                .start_in_program::<application::Program, program::ChainRoot>(&application)
                .unwrap()
        };
    }
    let mut left = root!("diamond-left");
    let mut right = root!("diamond-right");
    let mut other = root!("diamond-join");
    for row in [&mut left, &mut right, &mut other] {
        assert!(matches!(
            row.advance(&request).unwrap(),
            WorthQueryApplicationOutputDemandProgress::Settled(_)
        ));
    }
    let mut join = request
        .demand(ChainDemand("diamond-join".into()))
        .start_dependent_in_program::<application::Program, program::ChainConnection>(&application)
        .unwrap();
    assert!(matches!(
        join.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    for (rank, name, y) in [(0, "diamond-left", 5), (1, "diamond-right", 54)] {
        let selected = request
            .query(PlanarRead {
                body_key: name.into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: name.into(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_78d0_u64 + rank))
            .execute_performed::<application::Program, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
    }
    let schedule =
        computation::Schedule::new(["diamond-left".into(), "diamond-right".into()], None, false);
    let fault_owner = Arc::clone(&application);
    *schedule.owner_completion.lock().unwrap() = Some(Arc::new(move |_| {
        fault_owner.delay_next_output_readiness_delivery_for_test();
    }));
    computation::arm(Some(schedule));
    let result = join.advance(&request);
    computation::arm(None);
    assert!(matches!(
        result.unwrap(),
        WorthQueryApplicationOutputDemandProgress::Pending
    ));
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (2, 2)
    );
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
        primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test(),
        before
    );
    assert!(matches!(
        left.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert_eq!(
        application.undelivered_publication_counts_for_test(),
        (1, 1),
        "the first owner's delivery cannot take the other owner's successor"
    );
    let between = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
    assert!(matches!(
        other.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    assert_eq!(
        application.entered_required_selection_is_frozen_for_test(),
        Some(true)
    );
    assert_eq!(
        primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test(),
        between
    );
    assert!(matches!(
        right.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
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
}
