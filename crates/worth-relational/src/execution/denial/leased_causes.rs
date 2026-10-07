use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ChargedBytes, ExecutionScan, LeaseRequest, ScanOutcome};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionBudget, ExecutionPosture,
    ExecutionRequestPolicy,
};

use super::{tests::assert_both, *};

fn request(
    workers: usize,
    memory: u64,
    work: u64,
    determinism: DeterminismContract,
) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            determinism,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, work),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn actual_child_lease_refusals_survive_both_domain_doors() {
    let authority = crate::tests::support::test_execution_authority();
    let parent = authority
        .request_lease(request(1, 1024, 10, DeterminismContract::CanonicalBitwise))
        .expect("parent admitted");
    for (request, expected) in [
        (
            request(2, 1024, 10, DeterminismContract::CanonicalBitwise),
            Cause::WorkerLimitExceedsParent,
        ),
        (
            request(1, 1025, 10, DeterminismContract::CanonicalBitwise),
            Cause::MemoryLimitExceedsParent,
        ),
        (
            request(1, 1024, 11, DeterminismContract::CanonicalBitwise),
            Cause::WorkLimitExceedsParent,
        ),
        (
            request(
                1,
                1024,
                10,
                DeterminismContract::ContractEquivalent(EquivalenceContractId::new(1)),
            ),
            Cause::EquivalenceContractUnavailable,
        ),
    ] {
        let denial = parent
            .child(request)
            .expect_err("child exceeds parent contract");
        assert_both(
            || PacketExecutionStop::Execution {
                boundary: None,
                reason: MapStop::Admission(denial),
            },
            expected,
            None,
        );
    }
}

struct RetainedBytes(Vec<u8>);
impl ChargedBytes for RetainedBytes {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}

fn scan_stop(
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    retained: u64,
    declared: u64,
) -> PacketExecutionStop {
    let identity = PartitionIdentity::new(7);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())])
        .expect("canonical scan");
    match scan.run(
        lease,
        RetainedBytes(vec![0; usize::try_from(retained).unwrap()]),
        declared,
        0,
        0,
        0,
        |_, _, _| Ok::<_, MapKernelFailure<PacketBudgetDenial>>((RetainedBytes(Vec::new()), ())),
    ) {
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => PacketExecutionStop::Execution { boundary, reason },
        ScanOutcome::Complete { .. } => panic!("scan must stop before evaluating its step"),
    }
}

#[test]
fn actual_declared_memory_and_charge_overflow_survive_both_domain_doors() {
    let lease = crate::tests::support::test_execution_lease();
    assert_both(
        || scan_stop(Some(&lease), 11, 10),
        Cause::DeclaredMemoryExhausted {
            requested: 11,
            admitted: 10,
        },
        None,
    );
    assert_both(
        || scan_stop(Some(&lease), 0, u64::MAX),
        Cause::ChargedBytesOverflow,
        None,
    );
}

#[test]
fn actual_work_counter_overflow_survives_both_domain_doors() {
    let lease = crate::tests::support::test_execution_lease();
    assert_both(
        || {
            crate::execution::execute_read_only_packets(
                vec![super::super::ReadOnlyPacket {
                    identity: PartitionIdentity::new(7),
                    value: (),
                    input_bytes: 0,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                }],
                Some(&lease),
                |_, context| {
                    context.checkpoint(1)?;
                    context.account_completed_work(u64::MAX)?;
                    Ok::<_, MapKernelFailure<PacketBudgetDenial>>(())
                },
                |_| 0,
            )
            .expect_err("completed-work counter overflows")
        },
        Cause::WorkCounterOverflow,
        Some(7),
    );
}

#[test]
fn actual_unrelated_nested_lease_and_parent_stop_survive_both_domain_doors() {
    let lease = crate::tests::support::test_execution_lease();
    assert_both(
        || {
            crate::execution::execute_read_only_packets(
                vec![super::super::ReadOnlyPacket {
                    identity: PartitionIdentity::new(7),
                    value: (),
                    input_bytes: 0,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                }],
                Some(&lease),
                |_, _| {
                    assert_both(|| scan_stop(None, 0, 0), Cause::UnrelatedNestedLease, None);
                    Ok::<_, MapKernelFailure<PacketBudgetDenial>>(())
                },
                |_| 0,
            )
            .expect_err("ignored nested refusal stops parent")
        },
        Cause::NestedStopped,
        Some(7),
    );
}

#[test]
fn actual_packet_scratch_and_result_refusals_survive_both_domain_doors() {
    let lease = crate::tests::support::test_execution_lease();
    for (scratch, expected) in [
        (true, Cause::ScratchCapacityExceeded),
        (false, Cause::ResultCapacityExceeded),
    ] {
        assert_both(
            || {
                crate::execution::execute_read_only_packets(
                    vec![super::super::ReadOnlyPacket {
                        identity: PartitionIdentity::new(7),
                        value: (),
                        input_bytes: 0,
                        kernel_scratch_bytes: 0,
                        max_result_bytes: 0,
                    }],
                    Some(&lease),
                    |_, context| {
                        if scratch {
                            context.claim_scratch(1)?;
                        } else {
                            context.claim_result(1)?;
                        }
                        Ok::<_, MapKernelFailure<PacketBudgetDenial>>(())
                    },
                    |_| 0,
                )
                .expect_err("packet exceeds its declared capacity")
            },
            expected,
            Some(7),
        );
    }
}

#[test]
fn actual_packet_panic_survives_both_domain_doors() {
    let lease = crate::tests::support::test_execution_lease();
    assert_both(
        || {
            crate::execution::execute_read_only_packets(
                vec![super::super::ReadOnlyPacket {
                    identity: PartitionIdentity::new(7),
                    value: (),
                    input_bytes: 0,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                }],
                Some(&lease),
                |_, _| -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
                    panic!("packet panic is contained by the execution authority")
                },
                |_| 0,
            )
            .expect_err("execution contains the worker panic")
        },
        Cause::WorkerFailed,
        Some(7),
    );
}
