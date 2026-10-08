//! An index with no room to retain a commit stops the advance that met it.

use super::*;
use primary_graph::WorthQueryOutputDemandRecoveryPosture as Posture;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;

const WINDOW: usize = 8;

/// A demand stop: its kind, and what it tells the caller about asking again.
type Stop = (WorthQueryOutputDemandDenialKind, Posture);

/// What one index left of the chain's journey.
struct Journey {
    /// Largest actual owner reservation observed at the journey boundaries.
    retained_bytes: u64,
    /// Whether all genuine window-advancing mutations committed.
    window_fits: bool,
    /// Every stop an advance met.
    stops: Vec<Stop>,
    before_window: u64,
    after_window: u64,
}

/// Only the actual native-index capacity stop qualifies as a refused write.
fn assert_retained_refusal(
    no_effect: worth_query_host::facade::product::WorthQueryApplicationNoEffect,
) {
    use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause as Cause;
    use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalPublicationDeferred};
    assert!(
        matches!(
            no_effect.cause(),
            Cause::RelationalDeferred(RelationalPublicationDeferred::CompanionPreflight(
                CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
            ))
        ),
        "the native write refuses retained index capacity: {no_effect:?}"
    );
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
    use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation};
    let (application, invalidation) =
        limited_application(4 * 1_024 * 1_024, invalidation_bytes, WINDOW);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut stops = Vec::new();
    let mut retained_bytes = invalidation.retained_capacity_bytes();
    macro_rules! advance_recording {
        ($stops:expr, $demand:expr, $request:expr) => {{
            let progress = match $demand.advance(&$request) {
                Ok(progress) => Some(progress),
                Err(WorthQueryApplicationOutputDemandDenial::Demand(denial)) => {
                    $stops.push((denial.kind(), denial.recovery_posture()));
                    None
                }
                Err(other) => panic!("the advance stops as a demand: {other:?}"),
            };
            retained_bytes = retained_bytes.max(invalidation.retained_capacity_bytes());
            progress
        }};
    }
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
        let selected = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap();
        let changed = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_3e00_u64 + cycle))
            .execute_performed::<program::ChainProgram, program::ChainRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        use worth_query_host::facade::application_entry::{
            WorthQueryApplicationMutationOutcome as Outcome,
            WorthQueryApplicationPerformedMutationOutcome as Performed,
        };
        let written = match changed {
            Ok(Performed::Performed(_)) => true,
            Ok(Performed::NotPerformed(Outcome::Commit(
                primary_graph::WorthQueryApplicationUncommitted::NoEffect(no_effect),
            ))) => {
                assert_retained_refusal(no_effect);
                false
            }
            Ok(Performed::NotPerformed(other)) => {
                panic!("unexpected root mutation outcome: {other:?}")
            }
            Ok(Performed::RequiredOutputDenied { denial, .. }) => {
                panic!("required source custody is ample: {denial:?}")
            }
            Err(other) => panic!("unexpected root mutation denial: {other:?}"),
        };
        retained_bytes = retained_bytes.max(invalidation.retained_capacity_bytes());
        if !written {
            break;
        }
        refused = [
            advance_recording!(stops, d, request).is_none(),
            advance_recording!(stops, c, request).is_none(),
            advance_recording!(stops, b, request).is_none(),
            advance_recording!(stops, a, request).is_none(),
        ];
    }
    let before_window = invalidation.retained_capacity_bytes();
    let room = (0..WINDOW as u64).all(|position| {
        let y = 20 + position % 2;
        let selected = request.query(PlanarRead { body_key: "anchor-c".to_owned() })
            .execute().unwrap();
        // This unused native field advances the canonical retained window;
        // it needs no producer settlement or performed-output graph.
        let outcome = request.mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-c".to_owned(),
            operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "anchor-c".to_owned(), replacement_y: length(y),
            }]), validator_work: 4_096,
        })).expect_source(selected.observed_sources()[0].clone())
            .idempotency(&(0x9176_3e80_u64 + position))
            .execute_in_program::<program::ChainProgram>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
        use primary_graph::WorthQueryApplicationUncommitted as Uncommitted;
        let written = match outcome {
            Ok(Outcome::Committed { .. }) => true,
            Ok(Outcome::Commit(Uncommitted::NoEffect(no_effect))) => {
                assert_retained_refusal(no_effect);
                false
            },
            other => panic!("window mutation must commit or refuse retained capacity: {other:?}"),
        };
        retained_bytes = retained_bytes.max(invalidation.retained_capacity_bytes());
        written
    });
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
            }
        };
    }
    claim_again!(d, refused[0]);
    claim_again!(c, refused[1]);
    claim_again!(b, refused[2]);
    claim_again!(a, refused[3]);
    let after_window = invalidation.retained_capacity_bytes();
    drop((a, b, c, d));
    Journey {
        retained_bytes,
        window_fits: room,
        stops,
        before_window,
        after_window,
    }
}

#[test]
fn an_index_too_small_for_a_commit_stops_the_advance_for_retention() {
    let _guard = checkpoint_recovery_test_guard();
    // This is a logical reservation measurement, not a heap peak. The ample
    // control independently proves genuine native-window advancement releases
    // retained owner custody.
    let baseline = chain_journey(8 * 1_024 * 1_024);
    assert!(baseline.stops.is_empty(), "the measurement journey settles");
    assert!(baseline.window_fits, "the measurement window advances");
    assert!(
        baseline.retained_bytes > 0,
        "the owner retained a real index"
    );
    assert!(
        baseline.after_window < baseline.before_window,
        "the ample native window releases actual owner reservations"
    );

    // Finite profiles span half through twice the observed reservation. No
    // monotonicity or refusal/recovery overlap is assumed: a native window
    // write must also fund coexistence of old and new inherited roots.
    // The genuine registry-index held-address/root-retirement courts exercise
    // refusal/release/retry separately; that is a distinct capacity owner.
    let mut refused = 0;
    for sixteenths in 8..=32 {
        let capacity = baseline.retained_bytes.checked_mul(sixteenths).unwrap() / 16;
        let journey = chain_journey(capacity);
        refused += usize::from(!journey.stops.is_empty());
        for stop in journey.stops {
            assert_eq!(
                stop,
                (
                    WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                    Posture::Terminal,
                ),
                "{capacity} bytes of index: the refused advance stops for retention"
            );
        }
    }
    assert!(
        refused != 0,
        "some index in the sweep is too small for the chain"
    );
}
