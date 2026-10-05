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
///
/// Beside each order is what every advance costs: the producer contacts its
/// demand reports, the source queries it runs and the chain decisions it
/// makes. An advance produces its row and what that row reads. The root reads
/// its source once to be produced, and once more when its own demand is the
/// one that finds the edit. A demand whose row an earlier advance produced
/// follows it without a contact, and the root's advance decides the last
/// consumer when the middle one has just republished under it.
#[test]
fn a_held_chain_follows_an_edit_the_window_no_longer_retains() {
    let _guard = checkpoint_recovery_test_guard();
    for (order, follows) in [
        (['a', 'b', 'c'], [(1, 2, 0), (1, 1, 1), (1, 1, 1)]),
        (['c', 'b', 'a'], [(1, 3, 2), (0, 0, 0), (0, 0, 0)]),
        (['b', 'a', 'c'], [(1, 2, 1), (0, 1, 1), (0, 0, 0)]),
    ] {
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
        let held = order.map(|role| {
            let cost = match role {
                'a' => settled!(court, a, at),
                'b' => settled!(court, b, at),
                _ => settled!(court, c, at),
            };
            (
                cost.producer_contacts,
                cost.source_queries,
                judge_decisions(&mut rings, &at),
            )
        });
        assert_eq!(
            held, follows,
            "{at}: every advance follows the edit at its exact cost"
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

/// A demand joins a settled root and never advances. The root input is edited
/// and the window moves past the edit, so the joined demand stops superseded:
/// it named a source the commit replaced. That stop ends nothing its settled
/// holders keep. The held root and its consumer follow the output whichever
/// demand advances first, and the joined demand then follows the refreshed root.
#[test]
fn a_never_settled_stop_leaves_the_output_to_its_settled_holders() {
    let _guard = checkpoint_recovery_test_guard();
    for unsettled_first in [true, false] {
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
        let court = Court::new(&application, &request, 0x9176_3e80);
        let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
        take_all_decisions();
        let at = format!("unsettled first {unsettled_first}, before the commits");
        let mut a = root!(court, rings[2].key("a"), at);
        let mut b = consumer!(court, rings[2].key("b"), at);
        settled!(court, a, at);
        settled!(court, b, at);
        judge_decisions(&mut rings, &at);
        let mut joined = root!(court, rings[2].key("a"), at);

        let at = format!("unsettled first {unsettled_first}, after the window moved past the edit");
        rings[2].a_y = 5;
        court.write_y(&rings[2].key("a"), 5, &at);
        for y in [2, 3, 4, 5, 6, 2, 3, 4] {
            rings[1].a_y = y;
            court.write_y(&rings[1].key("a"), y, &at);
        }
        if unsettled_first {
            let stop = joined.demand.advance(court.request).map(|progress| {
                matches!(
                    progress,
                    WorthQueryApplicationOutputDemandProgress::Settled(_)
                )
            });
            assert!(
                matches!(
                    stop,
                    Err(WorthQueryApplicationOutputDemandDenial::Superseded)
                ),
                "{at}: the demand that never settled named a source the commit replaced: {stop:?}"
            );
        }
        let held = [settled!(court, a, at), settled!(court, b, at)];
        assert_eq!(
            (
                held.map(|cost| cost.producer_contacts),
                held.map(|cost| cost.source_queries),
                judge_decisions(&mut rings, &at)
            ),
            ([1, 1], [2, 1], 1),
            "{at}: the settled holders follow the output"
        );
        court.judge_middle(&rings[2], &at);
        assert!(
            settled!(court, joined, at).is_free(),
            "{at}: the joined demand follows the refreshed root"
        );
    }
}
