//! Queue claims acquire their selection only when their owner is inspected.
use super::*;

#[test]
fn three_claimed_records_select_after_their_predecessors_publish() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_7f01_u64);
    assert!(application.queued_required_work_for_test() >= 3);
    let before = query_entries();
    settled_in_one_advance!(d, request, "the unrelated caller");
    assert_eq!(
        query_entries() - before,
        3,
        "each claimed row is selected after the preceding publication moves its membership"
    );
    assert_eq!(
        application.queued_required_work_for_test(),
        1,
        "only the unselected newer publication hint remains queued"
    );
    let before = query_entries();
    settled_in_one_advance!(a, request, "the refreshed A");
    settled_in_one_advance!(b, request, "the refreshed B");
    settled_in_one_advance!(c, request, "the refreshed C");
    assert_eq!(
        query_entries(),
        before,
        "every claimed member already decided"
    );
}
