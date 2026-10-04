//! A root whose output was republished equal, and the demands that read it.

use super::*;

/// The root republishes an equal output without reaching its producer, so the
/// consumer's edge reads an older row of a certified-equal chain. When the
/// root's input then changes, that edge waits on the root's own progression
/// and the consumer decides over the new output.
#[test]
fn a_consumer_of_an_equally_republished_root_decides_again_when_the_root_changes() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let mut rings = vec![Ring::seeded(0)];
    take_all_decisions();
    let at = "the first settlement";
    {
        let mut a = root!(court, rings[0].key("a"), at);
        let mut b = consumer!(court, rings[0].key("b"), at);
        settled!(court, a, at);
        settled!(court, b, at);
    }
    assert_eq!(
        judge_decisions(&mut rings, at),
        1,
        "{at}: the consumer decides"
    );

    let at = "a fetched field the root input omits changes";
    rings[0].far_y = 11;
    court.write_y(&rings[0].key("source-c"), 11, at);
    {
        let mut a = root!(court, rings[0].key("a"), at);
        let republication = settled!(court, a, at);
        assert_eq!(
            (
                republication.producer_contacts,
                judge_decisions(&mut rings, at)
            ),
            (0, 0),
            "{at}: the root republishes an equal output and reaches no producer"
        );
    }

    let at = "the root input changes";
    rings[0].a_y = 5;
    court.write_y(&rings[0].key("a"), 5, at);
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    assert_eq!(
        judge_decisions(&mut rings, at),
        1,
        "{at}: the consumer decides once over the new root output"
    );
    court.judge_middle(&rings[0], at);
}

/// A held root demand rejoins the equal republication another demand
/// settled. A writer that is not the producer then overwrites the output, and
/// the held consumer's advance refreshes the root under the same source. The
/// held root demand follows that refresh instead of stopping superseded.
#[test]
fn a_held_root_follows_a_refresh_of_its_equal_republication() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<1>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f40);
    let mut rings = vec![Ring::seeded(0)];
    take_all_decisions();
    let at = "the first settlement";
    let mut a = root!(court, rings[0].key("a"), at);
    let mut b = consumer!(court, rings[0].key("b"), at);
    settled!(court, a, at);
    settled!(court, b, at);
    judge_decisions(&mut rings, at);

    let at = "a fetched field the root input omits changes";
    rings[0].far_y = 11;
    court.write_y(&rings[0].key("source-c"), 11, at);
    {
        let mut fresh = root!(court, rings[0].key("a"), at);
        assert_eq!(
            settled!(court, fresh, at).producer_contacts,
            0,
            "{at}: the root republishes an equal output and reaches no producer"
        );
    }
    let held = [settled!(court, b, at), settled!(court, a, at)];
    assert!(
        held.iter().all(|cost| cost.producer_contacts == 0),
        "{at}: the held demands rejoin the republication: {held:?}"
    );

    let at = "another writer overwrites the root output";
    court.overwrite_length(&rings[0].key("a"), 95, at);
    settled!(court, b, at);
    settled!(court, a, at);
    judge_decisions(&mut rings, at);
    court.judge_middle(&rings[0], at);
}
