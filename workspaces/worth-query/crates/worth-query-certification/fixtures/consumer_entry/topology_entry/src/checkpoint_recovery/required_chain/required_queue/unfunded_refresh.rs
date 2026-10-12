//! An unfunded queued refresh cannot starve an unrelated caller.
use super::*;

#[test]
fn a_queued_chain_no_advance_can_fund_never_starves_an_unrelated_caller() {
    let _guard = checkpoint_recovery_test_guard();
    // Room for three Ready rows per settled demand holds the settled chain
    // but not a refreshed generation: the queued chain fails on every
    // advance that tries it.
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let (application, _) = limited_application(4 * 3 * row, 128 * 1_024 * 1_024, 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(application, request);
    let inexact = inexact_deliveries();
    change_root_input!(request, application, 2, 0x9176_3a00_u64);
    let before_unrelated = query_entries();
    for attempt in 0..4 {
        let settled = settled_in_one_advance!(d, request, "the unrelated required demand");
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            1,
            "attempt {attempt}: the unrelated open demand retains only its initial execution"
        );
    }
    assert!(
        query_entries() > before_unrelated,
        "the unrelated advances attempted the queued chain"
    );
    assert_eq!(
        inexact_deliveries(),
        inexact,
        "every publication stays exact"
    );
    drop((a, b, c, d));
}

#[test]
fn dependents_of_a_failed_required_refresh_never_stay_pending() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let (application, _) = limited_application(4 * 3 * row, 128 * 1_024 * 1_024, 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_3b00_u64);
    settled_in_one_advance!(d, request, "the unrelated required demand");
    for attempt in 0..3 {
        for (name, pending) in [
            ("the last consumer", c.advance(&request)),
            ("the middle consumer", b.advance(&request)),
        ] {
            assert!(
                !matches!(
                    pending,
                    Ok(WorthQueryApplicationOutputDemandProgress::Pending)
                ),
                "attempt {attempt}: {name} is Pending with no work a later advance can finish"
            );
        }
    }
    drop((a, b, c, d));
}
