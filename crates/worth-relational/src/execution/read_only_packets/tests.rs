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
fn unleased_read_failure_returns_its_canonical_partition_and_stops_the_suffix() {
    let visited = AtomicUsize::new(0);
    for cause in [
        crate::execution::RelationalExecutionDenialCause::Cancelled,
        crate::execution::RelationalExecutionDenialCause::DeadlineElapsed,
        crate::execution::RelationalExecutionDenialCause::WorkExhausted,
    ] {
        visited.store(0, Ordering::Relaxed);
        let packets = (0..3)
            .map(|ordinal| ReadOnlyPacket {
                identity: PartitionIdentity::new(ordinal),
                value: ordinal,
                input_bytes: 0,
                memory: crate::execution::PacketMemory::Declared {
                    scratch: 0,
                    result: 0,
                },
            })
            .collect::<Vec<_>>();
        let result = crate::execution::execute_unleased_read_only_packets(&packets, |ordinal| {
            visited.fetch_add(1, Ordering::Relaxed);
            if *ordinal == 1 {
                Err(cause)
            } else {
                Ok(*ordinal)
            }
        });
        let denial: crate::indexes::data::DerivedIndexExecutionDenial =
            PacketExecutionStop::from(result.unwrap_err()).into();
        assert_eq!(denial.partition_identity, Some(1));
        assert_eq!(
            denial.kind,
            crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause)
        );
        assert_eq!(visited.load(Ordering::Relaxed), 2);
    }
}

#[test]
fn one_large_packet_observes_cancellation_mid_kernel_without_result() {
    let cancellation = CancellationSource::new();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(100_000)),
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
            memory: crate::execution::PacketMemory::Declared {
                scratch: 1024,
                result: 1024,
            },
        }],
        &lease,
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
                memory: crate::execution::PacketMemory::Declared {
                    scratch: 0,
                    result: 0,
                },
            }],
            lease,
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
                    ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), None, Some(5)),
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

#[test]
fn tracked_packet_claim_refuses_before_growth_and_releases_its_payer() {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    let authority = crate::tests::support::test_execution_authority();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, Some(16 * 1024), Some(100)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let grew = AtomicUsize::new(0);
    let refused = execute_read_only_packets(
        vec![ReadOnlyPacket {
            identity: PartitionIdentity::new(7),
            value: (),
            input_bytes: 0,
            memory: crate::execution::PacketMemory::Tracked,
        }],
        &lease,
        |_, context| {
            context.claim_result(32 * 1024)?;
            grew.fetch_add(1, Ordering::Relaxed);
            Ok(vec![0_u8; 32 * 1024])
        },
        |bytes| bytes.capacity() as u64,
    )
    .unwrap_err();
    let denial: crate::transactions::data::CommitExecutionDenial = refused.into();
    assert!(matches!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted {
            ancestor: 0,
            ..
        })
    ));
    assert_eq!(denial.partition_identity, Some(7));
    assert_eq!(grew.load(Ordering::Relaxed), 0);
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn tracked_packet_without_request_ceiling_keeps_actual_claims_and_counter_overflow_typed() {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    let lease = crate::tests::support::test_execution_lease();
    assert_eq!(lease.policy().budget().charged_memory_bytes(), None);
    let packet = || {
        vec![ReadOnlyPacket {
            identity: PartitionIdentity::new(9),
            value: (),
            input_bytes: 0,
            memory: crate::execution::PacketMemory::Tracked,
        }]
    };
    let values = execute_read_only_packets(
        packet(),
        &lease,
        |_, context| {
            context.claim_result(256)?;
            assert!(lease.charged_memory_bytes() >= 256);
            Ok(vec![3_u8; 256])
        },
        |bytes| bytes.capacity() as u64,
    )
    .unwrap();
    assert_eq!(values, vec![vec![3; 256]]);
    assert_eq!(lease.charged_memory_bytes(), 0);
    let refused = execute_read_only_packets(
        packet(),
        &lease,
        |_, context| {
            context.claim_result(u64::MAX)?;
            Ok(Vec::<u8>::new())
        },
        |bytes| bytes.capacity() as u64,
    )
    .unwrap_err();
    let denial: crate::transactions::data::CommitExecutionDenial = refused.into();
    assert_eq!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(Cause::ChargedBytesOverflow)
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}
