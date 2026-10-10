//! A settled demand whose refresh has published and still waits for its
//! readiness delivery when the next commit replaces its source.

use super::*;

/// The held root refreshes after an edit, and its readiness delivery is
/// delayed, so the advance answers `Pending`. A second edit lands before the
/// delivery. The demand settled before, so it keeps following its output: one
/// more advance settles it on the second edit, and its consumer follows.
#[test]
fn a_settled_demand_follows_its_output_past_an_undelivered_refresh() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3e40);
    let mut rings = vec![Ring::seeded(0)];
    Reading::decisions();
    let at = "the first settlement";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    judge_decisions(&mut rings, at);

    let at = "the refresh waits for its delivery";
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    application.delay_next_output_readiness_delivery_for_test();
    assert!(
        matches!(
            a.demand.advance(court.request),
            Ok(WorthQueryApplicationOutputDemandProgress::Pending)
        ),
        "{at}: an undelivered readiness leaves the refresh in flight"
    );

    let at = "a commit replaces the source of the undelivered refresh";
    rings[0].a_y = 6;
    court.write_y(&rings[0].key("a"), 6, at);
    let held = [settled!(court, a, at), settled!(court, b, at)];
    assert_eq!(
        (
            held.map(|cost| cost.producer_contacts),
            judge_decisions(&mut rings, at)
        ),
        ([1, 1], 1),
        "{at}: the settled holders follow the output"
    );
    court.judge_middle(&rings[0], at);
    let clean = [settled!(court, a, at), settled!(court, b, at)];
    assert!(
        clean.iter().all(Cost::is_free) && judge_decisions(&mut rings, at) == 0,
        "{at}: the followed chain is clean: {clean:?}"
    );
}

/// A second demand joins the row of the undelivered refresh and never
/// settles. The next commit replaces the source it named, so it stops
/// superseded. A settled demand that stays open keeps following its output:
/// the held root and its consumer settle on the second edit in one advance
/// each, whichever of the two root demands advances first, and the joined
/// demand then follows the refreshed root.
#[test]
fn a_never_settled_stop_leaves_an_undelivered_refresh_to_its_settled_holder() {
    let _guard = checkpoint_recovery_test_guard();
    for unsettled_first in [true, false] {
        let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let court = Court::new(&application, &request, 0x9176_3e60);
        let mut rings = vec![Ring::seeded(0)];
        Reading::decisions();
        let at = format!("unsettled first {unsettled_first}, the first settlement");
        let mut a = root!(court, rings[0].key("a"), at);
        let mut b = consumer!(court, rings[0].key("b"), at);
        settled!(court, a, at);
        settled!(court, b, at);
        judge_decisions(&mut rings, &at);

        let at =
            format!("unsettled first {unsettled_first}, a demand joins the undelivered refresh");
        rings[0].a_y = 5;
        court.write_y(&rings[0].key("a"), 5, &at);
        application.delay_next_output_readiness_delivery_for_test();
        assert!(
            matches!(
                a.demand.advance(court.request),
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            ),
            "{at}: an undelivered readiness leaves the refresh in flight"
        );
        let mut joined = root!(court, rings[0].key("a"), at);

        let at = format!("unsettled first {unsettled_first}, a commit replaces the joined source");
        rings[0].a_y = 6;
        court.write_y(&rings[0].key("a"), 6, &at);
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
                judge_decisions(&mut rings, &at)
            ),
            ([1, 1], 1),
            "{at}: the settled holders follow the output"
        );
        court.judge_middle(&rings[0], &at);
        assert!(
            settled!(court, joined, at).is_free(),
            "{at}: the joined demand follows the refreshed root"
        );
        let clean = [settled!(court, a, at), settled!(court, b, at)];
        assert!(
            clean.iter().all(Cost::is_free) && judge_decisions(&mut rings, &at) == 0,
            "{at}: the followed chain is clean: {clean:?}"
        );
    }
}
