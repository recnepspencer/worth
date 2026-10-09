//! Source writes commit within the ceiling; refused derived registrations recover.

use super::*;
use primary_graph::WorthQueryOutputDemandRecoveryPosture as Posture;
use support::capacity_region::{search, Attempt};
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;

/// Commits the retained window holds.
const WINDOW: usize = 8;

/// A demand stop: its kind, and what it tells the caller about asking again.
type Stop = (WorthQueryOutputDemandDenialKind, Posture);

/// What one index left of the chain's journey.
#[derive(Debug)]
struct Journey {
    /// Every stop an advance met.
    stops: Vec<Stop>,
    /// The rows a refused advance left that a later claim settled, once the
    /// window had moved past the journey's commits.
    reclaimed: usize,
    refused_consumers: usize,
    fresh_consumer_decisions: usize,
    before_window_bytes: u64,
    after_window_bytes: u64,
    peak_retained_bytes: u64,
    final_retained_bytes: u64,
}

/// Advances `demand` once. A demand stop is recorded.
macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr) => {{
        let advanced = $demand.advance(&$request);
        bounded!();
        match advanced {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                if $stops.is_empty()
                    && denial.kind() != WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                {
                    return Err(Attempt::Below(
                        "producer unavailable before index admission",
                    ));
                }
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
fn chain_journey(invalidation_bytes: u64) -> Result<Journey, Attempt> {
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
    let mut peak_retained_bytes = 0;
    macro_rules! bounded {
        () => {
            let retained = invalidation.retained_capacity_bytes();
            if retained > peak_retained_bytes {
                peak_retained_bytes = retained;
            }
            assert!(
                invalidation.retained_capacity_bytes() <= invalidation_bytes,
                "{invalidation_bytes}: retained {}",
                invalidation.retained_capacity_bytes()
            );
        };
    }
    settle_recording!(a);
    if !stops.is_empty() {
        return Err(Attempt::Below("initial upstream admission"));
    }
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
    let before_window_bytes = invalidation.retained_capacity_bytes();
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
    let after_window_bytes = invalidation.retained_capacity_bytes();
    assert!(after_window_bytes <= invalidation_bytes);
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
    Ok(Journey {
        stops,
        reclaimed,
        refused_consumers,
        fresh_consumer_decisions,
        before_window_bytes,
        after_window_bytes,
        peak_retained_bytes,
        final_retained_bytes: invalidation.retained_capacity_bytes(),
    })
}

fn index_attempt(capacity: usize, fresh: bool) -> Attempt {
    let journey = match chain_journey(capacity as u64) {
        Ok(journey) => journey,
        Err(answer) => return answer,
    };
    if (fresh && journey.refused_consumers == 0) || (!fresh && journey.stops.is_empty()) {
        return Attempt::Above("consumer registration admitted");
    }
    if !fresh {
        for stop in &journey.stops {
            assert_eq!(
                *stop,
                (
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    Posture::Terminal
                ),
                "an index-refused advance stops terminal for retention"
            );
        }
    }
    if fresh {
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
    Attempt::Hit
}

#[test]
fn an_index_too_small_for_a_commit_stops_the_advance_for_retention() {
    let _guard = checkpoint_recovery_test_guard();
    let ample_capacity = 128 * 1024 * 1024;
    let ample = chain_journey(ample_capacity).unwrap();
    assert!(ample.stops.is_empty(), "the ample chain has no stops");
    assert!(
        ample.peak_retained_bytes <= ample_capacity,
        "the native window fits"
    );
    assert!(
        ample.peak_retained_bytes > 0,
        "the owner retains a real index"
    );
    assert!(
        ample.after_window_bytes < ample.before_window_bytes,
        "the native window releases custody before any demand closes"
    );
    assert!(
        ample.final_retained_bytes < ample.peak_retained_bytes,
        "the complete journey releases retained reservations"
    );
    let mut reclaimed = 0;
    let mut cache = std::collections::BTreeMap::new();
    // Each sweep step finds its own band; every legal write and one-advance recovery stays asserted.
    for steps in (16..=64).rev() {
        let quantum = steps;
        let band = search(
            &format!("index sweep {steps}"),
            1,
            128 * 1024,
            support::capacity_region::Goal::UpperEdge,
            |units| {
                *cache
                    .entry(units * quantum)
                    .or_insert_with(|| index_attempt(units * quantum, false))
            },
        );
        let edge = band.require_hit(&format!(
            "index sweep {steps}, capacity units of {quantum} bytes"
        ));
        assert_eq!(cache.get(&(edge * quantum)), Some(&Attempt::Hit));
        reclaimed += chain_journey((edge * quantum) as u64).unwrap().reclaimed;
        // The ample side of the same journey admits every registration.
        assert_eq!(
            index_attempt((edge + 1) * quantum, false),
            Attempt::Above("consumer registration admitted")
        );
    }
    assert!(
        reclaimed != 0,
        "some swept index has room for a refused row once the window moves"
    );
}

#[test]
fn a_consumer_with_incomplete_registration_settles_by_a_fresh_decision_after_capacity_returns() {
    let _guard = checkpoint_recovery_test_guard();
    search(
        "fresh consumer capacity",
        1,
        2 * 1024 * 1024,
        support::capacity_region::Goal::Hit,
        |capacity| index_attempt(capacity, true),
    )
    .require_hit("fresh consumer registration");
}
