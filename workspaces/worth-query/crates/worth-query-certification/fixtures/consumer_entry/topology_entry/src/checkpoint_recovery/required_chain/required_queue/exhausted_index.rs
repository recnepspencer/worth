//! Source writes commit; an index too small for one refuses the advance.

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
    refused_advances: Vec<AdvanceObservation>,
    /// Every stop an advance met.
    stops: Vec<Stop>,
    refused_consumers: usize,
    unrecovered_consumers: usize,
    fresh_consumer_decisions: usize,
    before_window_bytes: u64,
    after_window_bytes: u64,
    peak_retained_bytes: u64,
    final_retained_bytes: u64,
}

/// Advances `demand` once. A demand stop is recorded.
#[derive(Debug)]
struct AdvanceObservation {
    decisions: Vec<String>,
    published: bool,
    producer_attempts: u64,
    member_attempts: Vec<u64>,
}
macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr, $observations:expr, $application:expr, $roots:expr) => {{
        let before_members: Vec<_> = $roots
            .iter()
            .map(|root| $application.producer_contacts_at_root_on_this_thread_for_test(*root))
            .collect();
        let before_attempts = $application.producer_contacts_on_this_thread_for_test();
        let before_decisions = binding::decisions_snapshot().len();
        let before = $request.retain_read().unwrap().selected_commit().clone();
        let advanced = $demand.advance(&$request);
        bounded!();
        match advanced {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                if $stops.is_empty()
                    && denial.kind() != WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                {
                    return Err(Attempt::Above("the first stop is not the index's refusal"));
                }
                $observations.push(AdvanceObservation {
                    member_attempts: $roots
                        .iter()
                        .zip(before_members)
                        .map(|(root, before)| {
                            $application.producer_contacts_at_root_on_this_thread_for_test(*root)
                                - before
                        })
                        .collect(),
                    producer_attempts: $application.producer_contacts_on_this_thread_for_test()
                        - before_attempts,
                    decisions: binding::decisions_snapshot()
                        .into_iter()
                        .skip(before_decisions)
                        .map(|(scope, _)| scope)
                        .collect(),
                    published: $request.retain_read().unwrap().selected_commit() != &before,
                });
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
/// demand is advanced once more and a further stop is recorded. Whether that
/// claim settles is not a law here: a native window write must also fund
/// coexistence of old and new inherited roots.
fn chain_journey(invalidation_bytes: u64) -> Result<Journey, Attempt> {
    observed_chain_journey(invalidation_bytes, false)
}

fn observed_chain_journey(invalidation_bytes: u64, observe: bool) -> Result<Journey, Attempt> {
    let (application, invalidation) =
        limited_application(4 * 1_024 * 1_024, invalidation_bytes, WINDOW);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let roots = if observe {
        ["anchor-a", "anchor-b", "anchor-c", "anchor-source-b"]
            .map(|body_key| {
                request
                    .query(PlanarRead {
                        body_key: body_key.to_owned(),
                    })
                    .execute()
                    .unwrap()
                    .observed_sources()[0]
                    .root_entity_for_test()
            })
            .to_vec()
    } else {
        Vec::new()
    };
    let mut stops = Vec::new();
    let mut refused_advances = Vec::new();
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
                match advance_recording!(
                    stops,
                    $demand,
                    request,
                    refused_advances,
                    application,
                    roots
                ) {
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
            advance_recording!(stops, d, request, refused_advances, application, roots).is_none(),
            advance_recording!(stops, c, request, refused_advances, application, roots).is_none(),
            advance_recording!(stops, b, request, refused_advances, application, roots).is_none(),
            advance_recording!(stops, a, request, refused_advances, application, roots).is_none(),
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
    let refused_consumers = refused[1..3].iter().filter(|value| **value).count();
    binding::take_all_decisions();
    let mut unrecovered_consumers = 0;
    macro_rules! claim_again {
        ($demand:expr, $refused:expr, $consumer:expr) => {
            if $refused {
                let settled = matches!(
                    advance_recording!(
                        stops,
                        $demand,
                        request,
                        refused_advances,
                        application,
                        roots
                    ),
                    Some(WorthQueryApplicationOutputDemandProgress::Settled(_))
                );
                unrecovered_consumers += usize::from($consumer && !settled);
            }
        };
    }
    claim_again!(d, refused[0], false);
    claim_again!(c, refused[1], true);
    claim_again!(b, refused[2], true);
    claim_again!(a, refused[3], false);
    drop((a, b, c, d));
    let fresh_consumer_decisions = binding::take_all_decisions()
        .into_iter()
        .filter(|(scope, _)| matches!(scope.as_str(), "anchor-b" | "anchor-c"))
        .count();
    Ok(Journey {
        refused_advances,
        stops,
        refused_consumers,
        unrecovered_consumers,
        fresh_consumer_decisions,
        before_window_bytes,
        after_window_bytes,
        peak_retained_bytes,
        final_retained_bytes: invalidation.retained_capacity_bytes(),
    })
}

/// A recorded journey's first stop is the index's own refusal; the recorder
/// answers any other first stop as no index journey. After it a consumer whose
/// upstream has no output is denied by the fixture's handler. A current-output
/// retention refusal without a requested-output witness reaches a kept
/// unavailable row through ExecutionDenied. Nothing else
/// stops an advance, and no stop offers a retry.
fn assert_index_stops(capacity: u64, stops: &[Stop]) {
    use WorthQueryOutputDemandDenialKind::{
        ProducerDomainDenied, ProducerUnavailable, RetentionBudgetExceeded,
    };
    for stop in stops {
        assert!(
            matches!(
                stop,
                (
                    RetentionBudgetExceeded | ProducerDomainDenied | ProducerUnavailable,
                    Posture::Terminal
                )
            ),
            "{capacity} bytes of index: an advance stops only for retention or \n             for what the refusal left unproduced: {stop:?}"
        );
    }
}

fn refused_index(capacity: usize) -> Attempt {
    match chain_journey(capacity as u64) {
        Ok(journey) if journey.stops.is_empty() => Attempt::Above("index admitted"),
        Ok(journey) => {
            assert_index_stops(capacity as u64, &journey.stops);
            Attempt::Hit
        }
        Err(answer) => answer,
    }
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
    // The owner evicts down to what the chain needs, so the ample peak says
    // nothing about where refusal begins. One search finds a refused index;
    // the sweep runs from half through twice that capacity. No monotonicity
    // or refusal/recovery overlap is assumed.
    let refusing = search(
        "refused index",
        1,
        usize::try_from(ample.peak_retained_bytes).unwrap(),
        support::capacity_region::Goal::Hit,
        refused_index,
    )
    .require_hit("refused index") as u64;
    let mut refused = 0;
    for sixteenths in 8..=32 {
        let capacity = refusing * sixteenths / 16;
        // Below the initial upstream's own admission there is no chain to refuse.
        let Ok(journey) = chain_journey(capacity) else {
            continue;
        };
        if !journey.stops.is_empty() {
            refused += 1;
            assert_index_stops(capacity, &journey.stops);
        }
    }
    assert!(
        refused != 0,
        "some index in the sweep is too small for the chain"
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
        |capacity| {
            let journey = match chain_journey(capacity as u64) {
                Ok(journey) => journey,
                Err(answer) => return answer,
            };
            if journey.refused_consumers == 0 {
                return Attempt::Above("consumer registration admitted");
            }
            if journey.unrecovered_consumers != 0 {
                return Attempt::Below("refused consumer does not recover after the window moves");
            }
            assert!(
                journey.refused_consumers > 0,
                "a consumed-output registration was refused"
            );
            assert_eq!(
                journey.unrecovered_consumers, 0,
                "every refused consumer recovers in one advance"
            );
            assert!(
                journey.fresh_consumer_decisions > 0,
                "recovery runs a fresh consumer decision"
            );
            Attempt::Hit
        },
    )
    .require_hit("fresh consumer registration");
}

mod attempts;
