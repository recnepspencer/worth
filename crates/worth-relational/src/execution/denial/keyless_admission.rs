//! Real keyless packet admission preserves the only representable refusals.
use super::{tests::assert_both, *};
use crate::execution::{execute_read_only_packets, ReadOnlyPacket};

fn packet(identity: u64, scratch: u64, result: u64) -> ReadOnlyPacket<()> {
    ReadOnlyPacket {
        identity: PartitionIdentity::new(identity),
        value: (),
        input_bytes: 0,
        kernel_scratch_bytes: scratch,
        max_result_bytes: result,
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
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1024, 10),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    for lease in [None, Some(&lease)] {
        assert_both(
            || {
                execute_read_only_packets(
                    vec![packet(2, 0, 0), packet(1, 0, 0)],
                    lease,
                    |_, _| Ok(()),
                    |_| 0,
                )
                .expect_err("identity ordering is admitted before the kernel")
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
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1024, 10),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    assert_both(
        || {
            execute_read_only_packets(
                vec![packet(1, u64::MAX, 0), packet(2, 1, 0)],
                Some(&lease),
                |_, _| Ok(()),
                |_| 0,
            )
            .expect_err("the partitions' scratch declarations cannot be summed")
        },
        Cause::MemoryOverflow,
        None,
    );
}
