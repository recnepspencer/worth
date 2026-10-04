//! An index with no room to retain a commit stops the advance that met it.

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
}

/// Advances `demand` once. A demand stop is recorded.
macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr) => {
        match $demand.advance(&$request) {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                $stops.push((denial.kind(), denial.recovery_posture()));
                None
            }
            Err(other) => panic!("the advance stops as a demand: {other:?}"),
        }
    };
}

/// Writes the Y of `$key`, and answers whether the index had room for it.
macro_rules! writes_y {
    ($request:expr, $application:expr, $key:expr, $y:expr, $idempotency:expr) => {{
        let selected = $request
            .query(PlanarRead {
                body_key: $key.to_owned(),
            })
            .execute()
            .unwrap();
        let changed = $request
            .mutate(PlanarSourceAdjustment {
                scope_key: $key.to_owned(),
                replacement_y: length($y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&$idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(&$application);
        matches!(
            changed,
            Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
        )
    }};
}

/// The chain's journey with `invalidation_bytes` of retained index: it first
/// settles, then the root input changes a few times, each change followed by
/// one advance of every demand. An input change the index refuses ends the
/// changes, since nothing is left to refresh.
///
/// The rows the last advances were refused are then claimed again. Nothing
/// a demand does frees index room: it returns as commits move the window. So
/// a field no source of the chain reads is written once for every position
/// of the window. Where the index has room for those writes, each refused row
/// settles in one more advance of its demand.
fn chain_journey(invalidation_bytes: u64) -> Journey {
    let (application, _invalidation) =
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
                    Some(WorthQueryApplicationOutputDemandProgress::Pending) => {}
                    _ => break,
                }
            }
        };
    }
    settle_recording!(a);
    settle_recording!(b);
    settle_recording!(c);
    settle_recording!(d);
    // Whether the last advance of d, c, b and a was refused.
    let mut refused = [false; 4];
    for cycle in 0..4_u64 {
        let y = 2 + cycle % 2;
        if !writes_y!(request, application, "anchor-a", y, 0x9176_3e00_u64 + cycle) {
            break;
        }
        refused = [
            advance_recording!(stops, d, request).is_none(),
            advance_recording!(stops, c, request).is_none(),
            advance_recording!(stops, b, request).is_none(),
            advance_recording!(stops, a, request).is_none(),
        ];
    }
    let room = (0..WINDOW as u64).all(|position| {
        let y = 20 + position % 2;
        writes_y!(
            request,
            application,
            "anchor-c",
            y,
            0x9176_3e80_u64 + position
        )
    });
    let mut reclaimed = 0;
    macro_rules! claim_again {
        ($demand:expr, $refused:expr) => {
            if $refused {
                let settled = matches!(
                    advance_recording!(stops, $demand, request),
                    Some(WorthQueryApplicationOutputDemandProgress::Settled(_))
                );
                assert!(
                    settled || !room,
                    "{invalidation_bytes} bytes of index: with room, a later claim of {} settles",
                    stringify!($demand)
                );
                reclaimed += usize::from(settled && room);
            }
        };
    }
    claim_again!(d, refused[0]);
    claim_again!(c, refused[1]);
    claim_again!(b, refused[2]);
    claim_again!(a, refused[3]);
    drop((a, b, c, d));
    Journey { stops, reclaimed }
}

#[test]
fn an_index_too_small_for_a_commit_stops_the_advance_for_retention() {
    let _guard = checkpoint_recovery_test_guard();
    // The sweep runs from the index the steady chain fits in down to one a
    // quarter its size, so some of them refuse a registration, and some the
    // delivery of a producer's own commit. Work and required custody are
    // ample throughout, so retention is the only budget an advance can
    // exhaust. No stop is the producer's: a row its advance could not refresh
    // stays for a later claim, and never fails for every caller.
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
