//! Compiled now; dispatched only after the reviewed Home-maintenance funding merge.
use super::*;
use host_observation::{checkpoint_causality, comparison, native_semantic, projection, requests};
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation as Location,
};

pub(super) fn run() {
    let path = tempfile::tempdir().unwrap();
    let serial = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    // A single new journal entry crosses the count threshold. Bytes cannot
    // trigger first for this bounded fixture. This is policy, not a byte grant.
    let limits = worth_store_application::ApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::new(1).unwrap(),
        std::num::NonZeroU64::MAX,
    );
    let homes: Vec<_> = [
        Location::Directory(path.path().to_path_buf()),
        Location::MemoryOnly,
    ]
    .into_iter()
    .map(|location| {
        host_observation::opening::open(
            location,
            opening::build(),
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
            None,
            limits,
            &execution,
        )
    })
    .collect();
    let mut map = comparison::HostOracleComparison::default();
    let compare_state = |map: &mut comparison::HostOracleComparison| {
        comparison::compare(
            map,
            &projection::observe(homes[0].runtime(), &execution),
            &projection::observe(homes[1].runtime(), &execution),
        )
        .unwrap();
    };
    compare_state(&mut map);
    // One explicit ordinary warmup, no retry-until-green loop: it acknowledges
    // the bootstrap prefix through the same automatic publication owner.
    for home in &homes {
        assert!(requests::set(home.runtime(), 71_000, 611)
            .receipt()
            .is_some());
    }
    compare_state(&mut map);
    let before = homes[0].inspect_checkpoint(&execution).unwrap();
    // Between the two observations the ONLY Home action is this mutation.
    // In particular, there is no checkpoint(), recover_checkpoint() or close().
    for home in &homes {
        assert!(requests::set(home.runtime(), 71_001, 612)
            .receipt()
            .is_some());
    }
    let after = homes[0].inspect_checkpoint(&execution).unwrap();
    let crossings = checkpoint_causality::compare_count_triggered_mutation(&before, &after, limits)
        .unwrap_or_else(|cause| {
            panic!("causal automatic checkpoint: {cause:?}; before={before:?}; after={after:?}")
        });
    assert!(!crossings.is_empty());
    compare_state(&mut map);
    let originals: Vec<_> = homes
        .iter()
        .map(|home| {
            let Resolution::RecoveredCommitted(original) =
                requests::resolve_original(home.runtime(), 71_001, 612).unwrap()
            else {
                panic!("the actual settled source receipt is required");
            };
            host_observation::native_semantic::NativeSemanticProjection::observe(
                original.semantic_result().semantic(),
            )
        })
        .collect();
    native_semantic::compare(map.identities_mut(), &originals[0], &originals[1]).unwrap();

    // Negative twins derive from the actual observations: an old acknowledged
    // prefix or a withheld acknowledgement cannot prove this mutation's trigger.
    assert!(matches!(
        checkpoint_causality::compare_count_triggered_mutation(&before, &before, limits),
        Err(checkpoint_causality::CheckpointCausalityDivergence::NoCountCrossing)
    ));
    let mut unacknowledged = after;
    for (a, b) in unacknowledged.journals.iter_mut().zip(&before.journals) {
        a.covered = b.covered;
    }
    assert!(matches!(
        checkpoint_causality::compare_count_triggered_mutation(&before, &unacknowledged, limits),
        Err(checkpoint_causality::CheckpointCausalityDivergence::CrossingNotAcknowledged { .. })
    ));
}
