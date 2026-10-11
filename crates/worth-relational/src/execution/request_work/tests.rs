use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Barrier,
    },
    time::{Duration, Instant},
};

use worth_execution::{
    CancellationSource, CancellationToken, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
};

use crate::execution::{
    execute_read_only_packets_with_budget, ReadOnlyPacket, RelationalExecutionDenialCause,
};
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};

use super::RequestWorkBudget;

fn parent(
    cancellation: CancellationToken,
    deadline: Option<Instant>,
) -> ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), 1024 * 1024, 100),
            ),
            cancellation,
            deadline,
        })
        .unwrap()
}

fn packet() -> Vec<ReadOnlyPacket<u64>> {
    vec![ReadOnlyPacket {
        identity: PartitionIdentity::new(0),
        value: 2,
        input_bytes: 0,
        kernel_scratch_bytes: 0,
        max_result_bytes: 0,
    }]
}

#[test]
fn remaining_work_child_observes_parent_cancellation_during_work() {
    let cancellation = CancellationSource::new();
    let lease = parent(cancellation.token(), None);
    let budget = RequestWorkBudget::new();
    let rendezvous = Barrier::new(2);
    let read = AtomicUsize::new(0);

    let result = std::thread::scope(|scope| {
        scope.spawn(|| {
            rendezvous.wait();
            cancellation.cancel();
            rendezvous.wait();
        });
        execute_read_only_packets_with_budget(
            packet(),
            Some(&lease),
            Some(&budget),
            |rows, context| {
                context.checkpoint(1)?;
                read.fetch_add(1, Ordering::Relaxed);
                rendezvous.wait();
                rendezvous.wait();
                for _ in 1..*rows {
                    context.checkpoint(1)?;
                    read.fetch_add(1, Ordering::Relaxed);
                }
                Ok(*rows)
            },
            |_| 0,
        )
    });

    assert_eq!(read.load(Ordering::Relaxed), 1);
    let denial = CommitExecutionDenial::from(result.unwrap_err());
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)
    );
}

#[test]
fn remaining_work_child_refuses_the_next_charge_after_parent_deadline() {
    let lease = parent(
        CancellationToken::new(),
        Some(Instant::now() - Duration::from_secs(1)),
    );
    let budget = RequestWorkBudget::new();
    let read = AtomicUsize::new(0);
    let result = execute_read_only_packets_with_budget(
        packet(),
        Some(&lease),
        Some(&budget),
        |rows, context| {
            context.checkpoint(1)?;
            read.fetch_add(1, Ordering::Relaxed);
            Ok(*rows)
        },
        |_| 0,
    );

    assert_eq!(read.load(Ordering::Relaxed), 0);
    let denial = CommitExecutionDenial::from(result.unwrap_err());
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::DeadlineElapsed)
    );
}
