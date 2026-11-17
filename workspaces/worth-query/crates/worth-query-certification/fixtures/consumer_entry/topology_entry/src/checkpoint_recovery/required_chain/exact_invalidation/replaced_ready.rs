//! A settled output whose row a demand of a newer source replaced, when that
//! demand ends before it publishes.

use super::*;

#[cfg(test)]
mod serial {
    use super::*;
    #[test]
    fn a_never_settled_stop_leaves_the_output_to_its_settled_holders() {
        let _equivalence =
            primary_graph::WorthQueryInvalidationEquivalenceForTest::arm_on_this_thread_for_test();
        let _guard = checkpoint_recovery_test_guard();
        for unsettled_first in [true, false] {
            let application = install(None, ring_world::seed::<3>);
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            let court = Court::new(&application, &request, 0x9176_3e80);
            let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
            Reading::decisions();
            let at = format!("unsettled first {unsettled_first}, before the commits");
            let mut a = root!(court, rings[2].key("a"), at);
            let mut b = consumer!(court, rings[2].key("b"), at);
            settled!(court, a, at);
            settled!(court, b, at);
            judge_decisions(&mut rings, &at);
            // Joining the unchanged Ready would already retain accepted output.
            // This source change admits a genuinely unexecuted successor instead.
            rings[2].far_y = 11;
            court.write_y(&rings[2].key("source-c"), 11, &at);
            let mut joined = root!(court, rings[2].key("a"), at);

            let at = format!("unsettled first {unsettled_first}, after unrelated publications");
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
                ([1, 1], [1, 1], 1),
                "{at}: the settled holders follow the output"
            );
            court.judge_middle(&rings[2], &at);
            assert!(
                settled!(court, joined, at).is_free(),
                "{at}: the joined demand follows the refreshed root"
            );
        }
    }
}
