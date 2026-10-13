//! Real leased cache storage, reuse, refusal and final-owner release.
use super::plan::{CandidateInputBasis, SharedCandidateInputs};
use crate::execution::{
    InvariantPacketAllocations, PacketBudgetDenial, RelationalExecutionDenialCause,
};
use crate::identity::data::{EntityId, PartitionId, VersionId};
use std::cell::Cell;
use std::num::NonZeroUsize;
use std::sync::Arc;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(
    bytes: u64,
    posture: ExecutionPosture,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                posture,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), bytes, 1_000_000),
            ),
            cancellation,
            deadline: None,
        })
        .unwrap()
}

#[test]
fn funded_cache_reuses_lazy_inputs_and_keeps_custody_after_packet_completion() {
    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        // A funded single-rule input is retained even without declared overlap.
        let lease = lease(1024 * 1024, posture, CancellationToken::new());
        let inputs = Arc::new(SharedCandidateInputs::new(false, Some(&lease)).unwrap());
        let counts = Cell::new(0);
        let fills = Cell::new(0);
        let first = inputs
            .touched_partitions(
                CandidateInputBasis::Enforcement,
                VersionId(1),
                || {
                    counts.set(counts.get() + 1);
                    Some(Some(8192))
                },
                |ids| {
                    fills.set(fills.get() + 1);
                    ids.extend((0..8192).map(PartitionId));
                    true
                },
            )
            .unwrap();
        let second = inputs
            .touched_partitions(
                CandidateInputBasis::Enforcement,
                VersionId(1),
                || panic!("cache reuse must not repeat the counting traversal"),
                |_| panic!("cache reuse must not gather again"),
            )
            .unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(first.len(), 8192);
        assert_eq!(first[0], PartitionId(0));
        assert_eq!(first[8191], PartitionId(8191));
        assert_eq!((counts.get(), fills.get()), (1, 1));
        assert_eq!(inputs.counters().touched_partition_gathers, 1);
        assert_eq!(inputs.counters().reuse_hits, 1);
        assert!(inputs.failure().is_none());
        drop((first, second));

        let retained = lease.charged_memory_bytes();
        assert!(retained >= 8192 * std::mem::size_of::<PartitionId>() as u64);
        let packet = InvariantPacketAllocations::new(&lease).unwrap();
        packet.scratch().claim(64 * 1024).unwrap();
        assert!(lease.charged_memory_bytes() > retained);
        drop(packet);
        assert_eq!(lease.charged_memory_bytes(), retained);
        let last_reader = inputs.clone();
        drop(inputs);
        assert_eq!(lease.charged_memory_bytes(), retained);
        drop(last_reader);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn cache_refusal_precedes_unfunded_gather_and_preserves_the_actual_lease_denial() {
    for (bytes, expected_fills) in [(128 * 1024, 0), (384 * 1024, 1)] {
        let lease = lease(bytes, ExecutionPosture::Automatic, CancellationToken::new());
        let inputs = Arc::new(SharedCandidateInputs::new(true, Some(&lease)).unwrap());
        let fills = Cell::new(0);
        let result = inputs.touched_partitions(
            CandidateInputBasis::Enforcement,
            VersionId(1),
            || Some(Some(65_536)),
            |ids| {
                fills.set(fills.get() + 1);
                ids.extend((0..65_536).map(PartitionId));
                true
            },
        );
        assert!(
            result.is_none(),
            "raw Vec and retained Arc must coexist under the lease"
        );
        assert_eq!(fills.get(), expected_fills);
        assert!(
            matches!(inputs.failure(), Some(PacketBudgetDenial::ReadRequest(
            RelationalExecutionDenialCause::PolicyMemoryExhausted { ancestor: 0, requested, admitted, .. }
        )) if requested > admitted)
        );
        assert!(inputs
            .entity(
                CandidateInputBasis::Enforcement,
                VersionId(1),
                EntityId::new(PartitionId::main(), 7, 1),
                || panic!("a refused cache must not contact another source"),
            )
            .is_none());
        drop(inputs);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

#[test]
fn cached_absence_is_successful_but_cancelled_materialization_retains_refusal() {
    let cancellation = CancellationSource::new();
    let lease = lease(1024 * 1024, ExecutionPosture::Serial, cancellation.token());
    let inputs = Arc::new(SharedCandidateInputs::new(true, Some(&lease)).unwrap());
    let id = EntityId::new(PartitionId::main(), 7, 1);
    assert!(inputs
        .entity(CandidateInputBasis::Enforcement, VersionId(1), id, || None)
        .is_none());
    assert!(inputs
        .entity(
            CandidateInputBasis::Enforcement,
            VersionId(1),
            id,
            || panic!("successful absence is cached")
        )
        .is_none());
    assert!(inputs.failure().is_none());
    cancellation.cancel();
    assert!(inputs
        .entity(
            CandidateInputBasis::Enforcement,
            VersionId(2),
            id,
            || panic!("cancelled cache growth must not contact a source")
        )
        .is_none());
    assert!(matches!(
        inputs.failure(),
        Some(PacketBudgetDenial::ReadRequest(
            RelationalExecutionDenialCause::Cancelled
        ))
    ));
    drop(inputs);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
