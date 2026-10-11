//! The real wave records a failed checkpoint on its newly issued successor.
use super::*;

#[test]
fn a_queued_checkpoint_failure_records_the_successor_not_its_predecessor() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(application, request);
    let selected = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let root = selected.observed_sources()[0].root_entity_for_test();
    let before = application.required_row_stops_for_root_for_test(root);
    assert_eq!(
        before.len(),
        1,
        "the settled root has one retained predecessor"
    );
    assert_eq!(before[0].1, None);
    change_root_input!(request, application, 2, 0x9176_7900_u64);
    application.leave_one_readiness_attempt_for_test();
    let settled = settled_in_one_advance!(d, request, "the unchanged independent caller");
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let after = application.required_row_stops_for_root_for_test(root);
    assert_eq!(
        after
            .iter()
            .find(|(key, _)| key == &before[0].0)
            .map(|(_, stop)| stop.clone()),
        Some(None),
        "the predecessor carries no checkpoint failure"
    );
    let successors: Vec<_> = after
        .iter()
        .filter(|(key, _)| key != &before[0].0)
        .collect();
    assert_eq!(
        successors.len(),
        1,
        "exactly one real successor checkpoint was issued"
    );
    assert_eq!(
        successors[0].1,
        Some(WorthQueryOutputDemandDenialKind::SchedulingRejected),
        "the successor owns the real checked readiness-counter refusal by value"
    );
    drop((a, b, c, d));
}
