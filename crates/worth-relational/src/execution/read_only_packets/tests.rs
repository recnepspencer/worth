use super::*;
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};

use worth_execution::{CancellationSource, CancellationToken, LeaseRequest, MapKernelStop};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn one_large_packet_observes_cancellation_mid_kernel_without_result() {
    let cancellation = CancellationSource::new();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), 1024 * 1024, 100_000),
            ),
            deadline: None,
            cancellation: cancellation.token(),
        })
        .expect("lease admitted");
    let visited = AtomicUsize::new(0);
    let result = execute_read_only_packets(
        vec![ReadOnlyPacket {
            identity: PartitionIdentity::new(0),
            value: 10_000_usize,
            input_bytes: 0,
            kernel_scratch_bytes: 1024,
            max_result_bytes: 1024,
        }],
        Some(&lease),
        |limit, context| {
            for index in 0..*limit {
                if index == 64 {
                    cancellation.cancel();
                }
                context.checkpoint(1)?;
                visited.fetch_add(1, Ordering::Relaxed);
            }
            Ok(visited.load(Ordering::Relaxed))
        },
        |_| 0,
    );
    assert!(matches!(
        result,
        Err(PacketExecutionStop::Execution {
            reason: MapStop::Failure {
                cause: MapKernelFailure::Stop(MapKernelStop::Cancelled),
                ..
            },
            ..
        })
    ));
    assert_eq!(visited.load(Ordering::Relaxed), 64);
}

#[test]
fn sequential_packet_maps_share_a_commit_work_ceiling() {
    fn packet_map(
        lease: &ExecutionResourceLease<'_>,
        budget: Option<&super::super::RequestWorkBudget>,
    ) -> Result<Vec<u64>, PacketExecutionStop> {
        execute_read_only_packets_with_budget(
            vec![ReadOnlyPacket {
                identity: PartitionIdentity::new(0),
                value: 7_u64,
                input_bytes: 0,
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            }],
            Some(lease),
            budget,
            |value, context| {
                context.checkpoint(2)?;
                Ok(*value)
            },
            |_| 0,
        )
    }

    for posture in [ExecutionPosture::Serial, ExecutionPosture::Automatic] {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 1024 * 1024, 5),
                ),
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .expect("lease admitted");
        assert_eq!(packet_map(&lease, None).unwrap(), vec![7]);
        assert_eq!(packet_map(&lease, None).unwrap(), vec![7]);

        let budget = super::super::RequestWorkBudget::new();
        assert_eq!(packet_map(&lease, Some(&budget)).unwrap(), vec![7]);
        assert!(matches!(
            packet_map(&lease, Some(&budget)),
            Err(PacketExecutionStop::Execution {
                reason: MapStop::WorkExhausted { .. },
                ..
            })
        ));
    }
}
