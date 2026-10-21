//! A queue closure cannot reopen its accepted caller; another root still runs.
use super::*;
use worth_query_host::facade::primary_graph::caller_pass_reports_on_this_thread_for_test as passes;

#[test]
fn a_queued_consumer_holds_at_its_accepted_caller_without_holding_an_independent_root() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_7c00);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    let at = "the initial chain and independent root";
    let mut a = root!(court, rings[2].key("a"), at);
    let mut b = consumer!(court, rings[2].key("b"), at);
    let mut c = consumer!(court, rings[2].key("c"), at);
    let mut independent = root!(court, rings[0].key("a"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    settled!(court, c, at);
    settled!(court, independent, at);
    judge_decisions(&mut rings, at);

    let at = "two marked closures after an input edit";
    rings[2].a_y = 5;
    court.write_y(&rings[2].key("a"), 5, at);
    rings[0].a_y = 3;
    court.write_y(&rings[0].key("a"), 3, at);
    for y in [2, 3, 4, 5, 6, 2, 3, 4] {
        rings[1].a_y = y;
        court.write_y(&rings[1].key("a"), y, at);
    }
    settled!(court, a, at);
    passes();
    assert!(application.queued_required_work_for_test() >= 2);
    let cost = settled!(court, b, at);
    assert_eq!(
        passes().len(),
        1,
        "the caller pass is not repeated by its queue"
    );
    assert_eq!(
        cost.source_queries, 2,
        "B and the independent root are inspected; C cannot disclose its source behind B"
    );
    assert!(
        application.queued_required_work_for_test() >= 1,
        "C's unacknowledged mark remains for a later advance"
    );
    assert_eq!(
        court.length(&rings[0].key("a")),
        rings[0].root_output(),
        "the independent root published while C's closure was held"
    );
    judge_decisions(&mut rings, at);
    let cost = settled!(court, c, at);
    assert_eq!(
        cost.source_queries, 1,
        "C discloses only in its own advance"
    );
    court.judge_chain(&rings[2], at);
}

#[test]
fn a_decided_caller_does_not_hold_a_queued_members_certified_predecessor() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_7c40);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    let at = "the initial held chain";
    let mut a = root!(court, rings[2].key("a"), at);
    let mut b = consumer!(court, rings[2].key("b"), at);
    let mut c = consumer!(court, rings[2].key("c"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    settled!(court, c, at);
    judge_decisions(&mut rings, at);

    let at = "B refreshed before A's caller pass";
    rings[2].a_y = 5;
    court.write_y(&rings[2].key("a"), 5, at);
    for y in [2, 3, 4, 5, 6, 2, 3, 4] {
        rings[1].a_y = y;
        court.write_y(&rings[1].key("a"), y, at);
    }
    let b_cost = settled!(court, b, at);
    assert_eq!(
        b_cost.source_queries, 2,
        "B refreshes A and itself, holding C"
    );
    assert_eq!(judge_decisions(&mut rings, at), 1);
    assert!(application.queued_required_work_for_test() > 0);
    passes();
    let a_cost = settled!(court, a, at);
    assert_eq!(passes().len(), 1, "A's caller pass is decided only once");
    assert_eq!(a_cost.producer_contacts, 0, "A is already current");
    assert_eq!(
        a_cost.source_queries, 1,
        "C verifies its source in A's advance; B certifies behind the decided A"
    );
    assert_eq!(
        judge_decisions(&mut rings, at),
        0,
        "equal B output cuts off C's decision"
    );
    assert!(
        settled!(court, c, at).is_free(),
        "C was completed in A's advance"
    );
    court.judge_chain(&rings[2], at);
}
