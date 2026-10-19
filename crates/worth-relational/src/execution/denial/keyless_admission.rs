//! Real keyless packet admission preserves the only representable refusals.
use super::{tests::assert_both, *};
use crate::execution::{execute_read_only_packets, ReadOnlyPacket};

fn packet(identity: u64, scratch: u64, result: u64) -> ReadOnlyPacket<()> {
    ReadOnlyPacket {
        identity: PartitionIdentity::new(identity),
        value: (),
        input_bytes: 0,
        memory: crate::execution::PacketMemory::Declared {
            scratch: scratch,
            result: result,
        },
    }
}

#[test]
fn noncanonical_identities_remain_typed_with_and_without_a_lease() {
    let authority = crate::tests::support::test_execution_authority();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    None,
                    Some(10),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    for lease in [None, Some(&lease)] {
        assert_both(
            || {
                let packets = vec![packet(2, 0, 0), packet(1, 0, 0)];
                if let Some(lease) = lease {
                    execute_read_only_packets(packets, lease, |_, _| Ok(()), |_| 0)
                        .expect_err("identity ordering is admitted before the kernel")
                } else {
                    PacketExecutionStop::Admission(
                        crate::execution::admit_read_only_packet_order(&packets)
                            .expect_err("identity ordering is admitted before the unleased kernel"),
                    )
                }
            },
            Cause::ExpectedIdentitiesNotCanonical,
            None,
        );
    }
}

#[test]
fn declared_map_bytes_overflow_before_execution_admission() {
    let authority = crate::tests::support::test_execution_authority();
    let lease = authority
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    None,
                    Some(10),
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    assert_both(
        || {
            execute_read_only_packets(
                vec![packet(1, u64::MAX, 0), packet(2, 1, 0)],
                &lease,
                |_, _| Ok(()),
                |_| 0,
            )
            .expect_err("the partitions' scratch declarations cannot be summed")
        },
        Cause::MemoryOverflow,
        None,
    );
}
