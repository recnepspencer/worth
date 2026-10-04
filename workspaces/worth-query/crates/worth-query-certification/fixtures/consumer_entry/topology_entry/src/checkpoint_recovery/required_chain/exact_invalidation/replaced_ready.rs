//! A settled output whose row a demand of a newer source replaced, when that
//! demand ends before it publishes.

use super::*;

/// A fetched field changes, and another caller demands the root at the new
/// source and closes without advancing. The settled root and its consumer
/// stay demanded: they answer again and reach no producer.
#[test]
fn a_held_chain_outlives_a_newer_demand_that_closes_unsettled() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f80);
    let mut rings = vec![Ring::seeded(0)];
    take_all_decisions();
    let at = "the first settlement";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    judge_decisions(&mut rings, at);

    let at = "a newer demand closes unsettled";
    rings[0].far_y = 11;
    court.write_y(&rings[0].key("source-c"), 11, at);
    drop(root!(court, rings[0].key("a"), at));
    let held = [settled!(court, b, at), settled!(court, a, at)];
    assert!(
        held.iter().all(|cost| cost.producer_contacts == 0),
        "{at}: the held demands settle again and reach no producer: {held:?}"
    );
    assert_eq!(judge_decisions(&mut rings, at), 0, "{at}: nothing decides");
    court.judge_middle(&rings[0], at);
}

/// Two rows of one source hold the occurrence when the newer demand comes:
/// the held chain rejoined an equal republication, a writer that is not the
/// producer overwrote the output, the consumer pumped the refresh under the
/// same source, and one of the root's two demands followed it while the other
/// lagged. The newer demand closes without advancing, and the root advances
/// before its consumer: the row it follows answers again, not the one it left.
#[test]
fn a_refreshed_chain_outlives_a_newer_demand_that_closes_unsettled() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3fa0);
    let mut rings = vec![Ring::seeded(0)];
    take_all_decisions();
    let at = "the first settlement";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    let mut lagging = root!(court, rings[0].key("a"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    settled!(court, lagging, at);

    let at = "the held chain rejoins an equal republication";
    rings[0].far_y = 11;
    court.write_y(&rings[0].key("source-c"), 11, at);
    {
        let mut fresh = root!(court, rings[0].key("a"), at);
        settled!(court, fresh, at);
    }
    settled!(court, b, at);
    settled!(court, a, at);
    settled!(court, lagging, at);

    let at = "the consumer pumps the refresh of an overwritten root";
    court.overwrite_length(&rings[0].key("a"), 90, at);
    settled!(court, b, at);
    settled!(court, a, at);
    judge_decisions(&mut rings, at);
    court.judge_middle(&rings[0], at);

    let at = "a newer demand closes unsettled";
    rings[0].far_y = 12;
    court.write_y(&rings[0].key("source-c"), 12, at);
    drop(root!(court, rings[0].key("a"), at));
    let held = [settled!(court, a, at), settled!(court, b, at)];
    assert!(
        held.iter().all(|cost| cost.producer_contacts == 0),
        "{at}: the held demands settle again and reach no producer: {held:?}"
    );
    assert_eq!(judge_decisions(&mut rings, at), 0, "{at}: nothing decides");
    court.judge_middle(&rings[0], at);
    assert_eq!(
        settled!(court, lagging, at).producer_contacts,
        0,
        "{at}: the demand that lagged follows the refreshed root"
    );
}

/// The newer demand is still open when the root input changes under it. It
/// never settled, so it stops superseded. The held root and its consumer
/// refresh from the current source, and the stopped demand follows them.
#[test]
fn a_held_chain_outlives_a_newer_demand_the_next_commit_supersedes() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3fc0);
    let mut rings = vec![Ring::seeded(0)];
    take_all_decisions();
    let at = "the first settlement";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    judge_decisions(&mut rings, at);

    let at = "a commit supersedes a newer unsettled demand";
    rings[0].far_y = 11;
    court.write_y(&rings[0].key("source-c"), 11, at);
    let mut newer = root!(court, rings[0].key("a"), at);
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    assert!(
        matches!(
            newer.demand.advance(court.request),
            Err(WorthQueryApplicationOutputDemandDenial::Superseded)
        ),
        "{at}: the demand named a source the commit replaced"
    );
    settled!(court, b, at);
    settled!(court, a, at);
    assert_eq!(
        judge_decisions(&mut rings, at),
        1,
        "{at}: the consumer decides once over the new root output"
    );
    court.judge_middle(&rings[0], at);
    assert_eq!(
        settled!(court, newer, at).producer_contacts,
        0,
        "{at}: the stopped demand follows the refreshed root"
    );
}

/// A held chain is not advanced while its root input is edited and more
/// commits follow than the window retains, so no mark of that edit is left.
/// Each held demand still follows its output in one advance, whichever is
/// advanced first, and both consumers decide once over the new root output.
#[test]
fn a_held_chain_follows_an_edit_the_window_no_longer_retains() {
    let _guard = checkpoint_recovery_test_guard();
    for order in [['a', 'b', 'c'], ['c', 'b', 'a'], ['b', 'a', 'c']] {
        let application = install(
            None,
            ring_world::seed::<3>,
            Retained {
                commit_positions: 4,
                ..Retained::AMPLE
            },
        );
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let court = Court::new(&application, &request, 0x9176_3ec0);
        let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
        take_all_decisions();
        let at = format!("held order {order:?}, before the commits");
        let mut a = root!(court, rings[2].key("a"), at);
        let mut b = consumer!(court, rings[2].key("b"), at);
        let mut c = consumer!(court, rings[2].key("c"), at);
        settled!(court, a, at);
        settled!(court, b, at);
        settled!(court, c, at);
        judge_decisions(&mut rings, &at);

        let at = format!("held order {order:?}, after the window moved past the edit");
        rings[2].a_y = 5;
        court.write_y(&rings[2].key("a"), 5, &at);
        for y in [2, 3, 4, 5, 6, 2, 3, 4] {
            rings[1].a_y = y;
            court.write_y(&rings[1].key("a"), y, &at);
        }
        let held = order.map(|role| match role {
            'a' => settled!(court, a, at),
            'b' => settled!(court, b, at),
            _ => settled!(court, c, at),
        });
        assert!(
            held.iter().all(|cost| cost.producer_contacts <= 1)
                && judge_decisions(&mut rings, &at) == 2,
            "{at}: no producer is contacted twice and both consumers decide once: {held:?}"
        );
        court.judge_chain(&rings[2], &at);
        let clean = [
            settled!(court, a, at),
            settled!(court, b, at),
            settled!(court, c, at),
        ];
        assert!(
            clean.iter().all(Cost::is_free) && judge_decisions(&mut rings, &at) == 0,
            "{at}: the followed chain is clean: {clean:?}"
        );
    }
}
