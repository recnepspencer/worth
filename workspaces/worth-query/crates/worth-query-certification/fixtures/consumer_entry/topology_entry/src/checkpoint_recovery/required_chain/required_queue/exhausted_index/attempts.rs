//! Real refused advances count every chain member at the handler boundary.
use super::*;

#[test]
fn a_refused_consumer_registration_performs_each_chain_member_once() {
    let _guard = checkpoint_recovery_test_guard();
    // A and D are roots; B and C each make one authoritative chain decision.
    let chain = ["anchor-a", "anchor-b", "anchor-c", "anchor-source-b"];
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
