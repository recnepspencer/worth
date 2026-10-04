//! An index with no room to retain a commit stops the advance that met it.

use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;

/// Advances `demand` once. A demand stop is recorded by kind.
macro_rules! advance_recording {
    ($stops:expr, $demand:expr, $request:expr) => {
        match $demand.advance(&$request) {
            Ok(progress) => Some(progress),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                $stops.push(denial.kind());
                None
            }
            Err(other) => panic!("the advance stops as a demand: {other:?}"),
        }
    };
}

/// Every stop the chain's advances meet with `invalidation_bytes` of retained
/// index: while the chain first settles, then over a few changes of the root
/// input, each followed by one advance of every demand. An input change the
/// index refuses ends the journey, since nothing is left to refresh.
fn chain_stops(invalidation_bytes: u64) -> Vec<WorthQueryOutputDemandDenialKind> {
    let (application, _invalidation) =
        limited_application(4 * 1_024 * 1_024, invalidation_bytes, 8);
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
    for cycle in 0..4_u64 {
        let selected = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap();
        let changed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(2 + cycle % 2),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_3e00_u64 + cycle))
            .execute_performed::<program::ChainProgram, program::ChainRoot>(&application);
        if !matches!(
            changed,
            Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
        ) {
            break;
        }
        advance_recording!(stops, d, request);
        advance_recording!(stops, c, request);
        advance_recording!(stops, b, request);
        advance_recording!(stops, a, request);
    }
    drop((a, b, c, d));
    stops
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
    let step = 128 * 1_024_u64;
    let mut refused = 0_usize;
    for capacity in (16..=64_u64).rev().map(|steps| steps * step) {
        let stops = chain_stops(capacity);
        refused += usize::from(!stops.is_empty());
        for stop in stops {
            assert_eq!(
                stop,
                WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                "{capacity} bytes of index: an advance the index refuses stops for retention"
            );
        }
    }
    assert!(
        refused != 0,
        "some index in the sweep is too small for the chain"
    );
}
