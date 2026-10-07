//! Source writes commit within the ceiling; refused derived registrations recover.

use super::*;
use primary_graph::WorthQueryOutputDemandRecoveryPosture as Posture;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;

/// Commits the retained window holds.
const WINDOW: usize = 8;

/// A demand stop: its kind, and what it tells the caller about asking again.
type Stop = (WorthQueryOutputDemandDenialKind, Posture);

/// What one index left of the chain's journey.
struct Journey {
    /// Every stop an advance met.
    stops: Vec<Stop>,
    /// The rows a refused advance left that a later claim settled, once the
    /// window had moved past the journey's commits.
    reclaimed: usize,
    refused_consumers: usize,
    fresh_consumer_decisions: usize,
}

/// Advances `demand` once. A demand stop is recorded.
macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr) => {{
        let advanced = $demand.advance(&$request);
        bounded!();
        match advanced {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                $stops.push((denial.kind(), denial.recovery_posture()));
                None
            }
            Err(other) => panic!("the advance stops as a demand: {other:?}"),
        }
    }};
}

/// Settle the chain, change its root four times, and advance every demand.
/// Source writes always commit, even when the derived index is evicted.
/// After unrelated writes move through the retained window, every refused
/// row must settle in exactly one more advance.
fn chain_journey(invalidation_bytes: u64) -> Journey {
    let (application, invalidation) =
        limited_application(4 * 1_024 * 1_024, invalidation_bytes, WINDOW);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut stops = Vec::new();
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut d = request
        .demand(PlanarOutputDemand::new("anchor-source-b"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    macro_rules! settle_recording {
        ($demand:expr) => {
            for _ in 0..256 {
                match advance_recording!(stops, $demand, request) {
                    Some(WorthQueryApplicationOutputDemandProgress::Pending) => {
                        bounded!();
                    }
                    _ => {
                        bounded!();
                        break;
                    }
                }
            }
        };
    }
    macro_rules! bounded {
        () => {
            assert!(
                invalidation.retained_capacity_bytes() <= invalidation_bytes,
                "{invalidation_bytes}: retained {}",
                invalidation.retained_capacity_bytes()
            );
        };
    }
    settle_recording!(a);
    bounded!();
    settle_recording!(b);
    settle_recording!(c);
    settle_recording!(d);
    // Whether the last advance of d, c, b and a was refused.
    let mut refused = [false; 4];
    for cycle in 0..4_u64 {
        let y = 2 + cycle % 2;
        writes_y!(request, application, "anchor-a", y, 0x9176_3e00_u64 + cycle);
        bounded!();
        refused = [
            advance_recording!(stops, d, request).is_none(),
            advance_recording!(stops, c, request).is_none(),
            advance_recording!(stops, b, request).is_none(),
            advance_recording!(stops, a, request).is_none(),
        ];
    }
    for position in 0..WINDOW as u64 {
        let y = 20 + position % 2;
        writes_y!(
            request,
            application,
            "anchor-c",
            y,
            0x9176_3e80_u64 + position
        );
        bounded!();
    }
    assert!(invalidation.retained_capacity_bytes() <= invalidation_bytes);
    let refused_consumers = refused[1..3].iter().filter(|refused| **refused).count();
    binding::take_all_decisions();
    let mut reclaimed = 0;
    macro_rules! claim_again {
        ($demand:expr, $refused:expr) => {
            if $refused {
                let advanced = advance_recording!(stops, $demand, request);
                let settled = matches!(
                    advanced,
                    Some(WorthQueryApplicationOutputDemandProgress::Settled(_))
                );
                assert!(
                    settled,
                    "{invalidation_bytes} bytes of index: a later claim of {} settles; stops={stops:?}",
                    stringify!($demand),
                );
                bounded!();
                reclaimed += usize::from(settled);
            }
        };
    }
    claim_again!(d, refused[0]);
    claim_again!(c, refused[1]);
    claim_again!(b, refused[2]);
    claim_again!(a, refused[3]);
    drop((a, b, c, d));
    let fresh_consumer_decisions = binding::take_all_decisions()
        .into_iter()
        .filter(|(scope, _)| matches!(scope.as_str(), "anchor-b" | "anchor-c"))
        .count();
    Journey {
        stops,
        reclaimed,
        refused_consumers,
        fresh_consumer_decisions,
    }
}

#[test]
fn an_index_too_small_for_a_commit_stops_the_advance_for_retention() {
    let _guard = checkpoint_recovery_test_guard();
    // The sweep runs from the index the steady chain fits in down to one a
    // quarter its size. Every source write commits; derived registration or
    // demand verification can stop for retention. Work and required custody
    // are ample throughout. A row its advance could not refresh stays for a
    // later claim, and every capacity settles once the window releases bytes.
    //
    // The stop is terminal for its caller, as retention is wherever no
    // advance or close of a demand frees the room: index room returns only
    // as later commits move the window.
    let step = 128 * 1_024_u64;
    let (mut refused, mut reclaimed) = (0_usize, 0_usize);
    for capacity in (16..=64_u64).rev().map(|steps| steps * step) {
        let journey = chain_journey(capacity);
        refused += usize::from(!journey.stops.is_empty());
        reclaimed += journey.reclaimed;
        for stop in journey.stops {
            assert_eq!(
                stop,
                (
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    Posture::Terminal
                ),
                "{capacity} bytes of index: an advance the index refuses stops for retention"
            );
        }
    }
    assert!(
        refused != 0,
        "some index in the sweep is too small for the chain"
    );
    assert!(
        reclaimed != 0,
        "some index in the sweep refuses a row and has room once the window moves"
    );
}

#[test]
fn a_consumer_with_incomplete_registration_settles_by_a_fresh_decision_after_capacity_returns() {
    let _guard = checkpoint_recovery_test_guard();
    let journey = chain_journey(2 * 1_024 * 1_024);
    assert!(
        journey.refused_consumers > 0,
        "a consumed-output registration ran out of retained capacity"
    );
    assert!(
        journey.fresh_consumer_decisions > 0,
        "settling invokes the real consumer handler again"
    );
    assert!(journey.reclaimed >= journey.refused_consumers);
}
