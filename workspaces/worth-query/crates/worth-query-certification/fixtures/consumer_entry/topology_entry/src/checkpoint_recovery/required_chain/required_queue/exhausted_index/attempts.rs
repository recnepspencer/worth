//! Real refused advances count every chain member at the handler boundary.
use super::*;

#[test]
fn a_refused_consumer_registration_performs_each_chain_member_once() {
    let _guard = checkpoint_recovery_test_guard();
    // A and D are roots; B and C each make one authoritative chain decision.
    let chain = CHAIN;
    let consumers = &chain[1..chain.len() - 1];
    let ample = chain_journey(128 * 1024 * 1024).unwrap();
    search(
        "one attempt at refused registration",
        1,
        ample.peak_retained_bytes as usize,
        support::capacity_region::Goal::Hit,
        |capacity| {
            let journey = match observed_chain_journey(capacity as u64, true) {
                Ok(journey) => journey,
                Err(answer) => return answer,
            };
            if journey.stops.is_empty() {
                return Attempt::Above("index admitted");
            }
            assert_index_stops(capacity as u64, &journey.stops);
            let advances: Vec<_> = journey
                .refused_advances
                .iter()
                .filter(|advance| advance.published && !advance.decisions.is_empty())
                .collect();
            if advances.is_empty() {
                return Attempt::Below("no post-effect consumer refusal");
            }
            for advance in advances {
                assert_eq!(advance.member_attempts.len(), chain.len());
                assert_eq!(
                    advance.member_attempts.iter().sum::<u64>(),
                    advance.producer_attempts
                );
                for (member, attempts) in chain.iter().zip(&advance.member_attempts) {
                    assert!(
                        *attempts <= 1,
                        "{capacity}: member {member} performed {attempts} times"
                    );
                }
                // The observer counts real handler entries for roots and consumers.
                // Each handler can publish at most once, so the chain length bounds
                // authoritative publications, independently of completed decisions.
                assert!(
                    advance.producer_attempts <= chain.len() as u64,
                    "{capacity}: too many authoritative attempts: {advance:?}"
                );
                // Each of these handlers constructs at most one candidate, so
                // one decision per member bounds authoritative publications too.
                assert!(
                    advance.decisions.len() <= consumers.len(),
                    "{capacity}: {advance:?}"
                );
                for member in consumers {
                    assert!(
                        advance
                            .decisions
                            .iter()
                            .filter(|scope| scope == member)
                            .count()
                            <= 1,
                        "{capacity}: member {member} was selected twice: {advance:?}"
                    );
                }
            }
            Attempt::Hit
        },
    )
    .require_hit("post-effect consumer registration refusal");
}

/// A handler that stops before publication is not a performed member. This
/// separate proof counts real commits after registration has actually refused.
#[test]
fn a_refused_registration_cannot_publish_a_chain_member_twice_in_one_advance() {
    let _guard = checkpoint_recovery_test_guard();
    let ample = chain_journey(128 * 1024 * 1024).unwrap();
    search(
        "publications at refused registration",
        1,
        ample.peak_retained_bytes as usize,
        support::capacity_region::Goal::Hit,
        |capacity| {
            let journey = match observed_chain_journey(capacity as u64, true) {
                Ok(journey) => journey,
                Err(answer) => return answer,
            };
            if journey.stops.is_empty() {
                return Attempt::Above("index admitted");
            }
            assert_index_stops(capacity as u64, &journey.stops);
            let advances: Vec<_> = journey
                .refused_advances
                .iter()
                .filter(|advance| {
                    advance.member_publications.iter().any(|count| *count > 0)
                        && !advance.decisions.is_empty()
                })
                .collect();
            if advances.is_empty() {
                return Attempt::Below("no post-effect consumer refusal");
            }
            for advance in advances {
                assert_eq!(advance.member_publications.len(), CHAIN.len());
                for (member, publications) in CHAIN.iter().zip(&advance.member_publications) {
                    assert!(
                        *publications <= 1,
                        "{capacity}: member {member} published {publications} times"
                    );
                }
            }
            Attempt::Hit
        },
    )
    .require_hit("post-effect consumer registration refusal");
}

/// This public settlement reaches the production publication handoff, not a
/// direct call to the record. A no-op there must leave the seal count behind.
#[test]
fn a_producer_publication_records_its_member_before_checkpointing() {
    let _guard = checkpoint_recovery_test_guard();
    let (application, _) = limited_application(4 * 1024 * 1024, 128 * 1024 * 1024, WINDOW);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let root = request
        .query(PlanarRead {
            body_key: CHAIN[0].to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .root_entity_for_test();
    let before = application.producer_publications_at_root_on_this_thread_for_test(root);
    let recorded_before =
        application.producer_recorded_publications_at_root_on_this_thread_for_test(root);
    let mut demand = request
        .demand(PlanarOutputDemand::new(CHAIN[0]))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    assert!(matches!(
        demand.settle(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let publications =
        application.producer_publications_at_root_on_this_thread_for_test(root) - before;
    assert!(
        publications > 0,
        "the production handoff must have published"
    );
    assert_eq!(
        application.producer_recorded_publications_at_root_on_this_thread_for_test(root)
            - recorded_before,
        publications,
        "every production publication must seal its actual member before checkpointing"
    );
}
