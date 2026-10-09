//! Repeated real cancellations supersede rows without keeping their tickets.
use super::*;

#[test]
fn cancelled_refresh_rows_return_to_the_same_live_custody_and_ledger() {
    let _guard = checkpoint_recovery_test_guard();
    let row = primary_graph::required_ready_custody_bytes_for_test();
    let (application, invalidation) = limited_application(13 * row, 128 * 1_024 * 1_024, 8);
    let _ = application.consumed_output_custody_for_test();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c) = chain!(application, request);
    change_root_input!(request, application, 2, 0x612_7100_u64);
    let mut baseline = None;
    let mut selected = None;
    for attempt in 0..4 {
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let cancelled_scope = authentication::WorthQueryRequestScope::new(
            Instant::now() + Duration::from_secs(120),
            cancellation.token(),
        );
        let cancelled = application.request(&principal, &cancelled_scope);
        take_decisions("anchor-b");
        super::super::super::binding::cancel_during_next_decision("anchor-b", cancellation);
        let stopped = b.advance(&cancelled);
        assert!(
            matches!(&stopped,
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled),
            "attempt {attempt}: the real refresh was cancelled: {:?}",
            stopped.as_ref().err()
        );
        assert_eq!(
            take_decisions("anchor-b").len(),
            1,
            "the cancellation ran inside B"
        );
        // The first attempt refreshes A. Every later attempt has that same
        // selected truth, the same old B Ready and a cancelled new B row.
        let head = request.retain_read().unwrap();
        let commit = head.selected_commit().clone();
        assert_eq!(selected.get_or_insert(commit.clone()), &commit);
        let retained = (
            application.required_custody_bytes_for_test(),
            invalidation.retained_capacity_bytes(),
            application.output_lineage_retained_bytes_for_test(),
            application.consumed_output_custody_for_test(),
        );
        assert_eq!(
            *baseline.get_or_insert(retained),
            retained,
            "attempt {attempt}: replacing the cancelled row refunds its live custody and tickets"
        );
    }
    let settled = settled_in_one_advance!(b, request, "the cancelled refresh retry");
    assert_eq!(settled.producer_contacts_in_this_demand(), 6);
    settled_in_one_advance!(c, request, "the held downstream after retry");
    settled_in_one_advance!(a, request, "the refreshed root");
}
