//! Artifact preparation retains original request control and allocation custody.
use super::*;
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn an_artifact_reserves_its_declared_allocation_once() {
    const ALLOCATION: u64 = 20_000;
    let authority = crate::tests::support::test_execution_authority();
    let policy = |bytes| {
        ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(bytes), Some(100_000)),
        )
    };
    let request = |bytes| LeaseRequest {
        policy: policy(bytes),
        deadline: None,
        cancellation: CancellationToken::new(),
    };
    let quote_lease = authority.request_lease(request(8 * 1024 * 1024)).unwrap();
    let scan = crate::execution::admit_ordered_scan(vec![(PartitionIdentity::new(1), ())]).unwrap();
    let required = scan.declared_memory_requirement_for_lease::<(), PreparedArtifact<Vec<u8>>, super::super::PacketBudgetDenial>(
        &quote_lease, 0, ALLOCATION, 0, 0).unwrap();
    drop(quote_lease);
    for under in [false, true] {
        let lease = authority
            .request_lease(request(required - u64::from(under)))
            .unwrap();
        let mut began = false;
        let result = prepare_borrowed_artifact(&lease, None, ALLOCATION, |budget| {
            began = true;
            budget.claim(ALLOCATION)?;
            let mut buffer: Vec<u8> = Vec::new();
            buffer.try_reserve_exact(ALLOCATION as usize).unwrap();
            Ok((buffer, ALLOCATION))
        });
        if under {
            assert!(!began, "one byte under refuses before allocating");
            assert!(matches!(
                result,
                Err(PacketExecutionStop::Execution {
                    reason: worth_execution::MapStop::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(_)
                    ),
                    ..
                })
            ));
        } else {
            assert_eq!(result.unwrap().capacity(), ALLOCATION as usize);
            assert!(began);
        }
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}

fn tracked_lease(
    work: Option<u64>,
    cancellation: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, work),
            ),
            deadline,
            cancellation,
        })
        .unwrap()
}

#[test]
fn tracked_artifacts_use_the_original_request_and_shared_remaining_work() {
    let lease = tracked_lease(Some(3), CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let work = crate::execution::RequestWorkBudget::new();
    let first = prepare_tracked_artifact(&execution, &lease, Some(&work), |budget| {
        budget.checkpoint(2)?;
        Ok((17_u64, 0))
    })
    .unwrap();
    assert_eq!(first, 17);
    let second = prepare_tracked_artifact(&execution, &lease, Some(&work), |budget| {
        budget.checkpoint(2)?;
        Ok((19_u64, 0))
    });
    let denial = crate::transactions::data::CommitExecutionDenial::from(second.unwrap_err());
    assert_eq!(
        denial.kind,
        crate::transactions::data::CommitExecutionDenialKind::Cause(
            crate::execution::RelationalExecutionDenialCause::WorkExhausted
        )
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn tracked_artifacts_keep_original_cancellation_deadline_and_release_scope_memory() {
    use crate::execution::RelationalExecutionDenialCause as Cause;
    use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};
    use std::{
        cell::Cell,
        time::{Duration, Instant},
    };
    for expired in [false, true] {
        let cancellation = worth_execution::CancellationSource::new();
        let deadline = expired.then(|| Instant::now() - Duration::from_secs(1));
        let lease = tracked_lease(None, cancellation.token(), deadline);
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        let work = crate::execution::RequestWorkBudget::new();
        let began = Cell::new(false);
        let outcome = prepare_tracked_artifact(&execution, &lease, Some(&work), |budget| {
            began.set(true);
            cancellation.cancel();
            budget.checkpoint(1)?;
            Ok((23_u64, 0))
        });
        let denial = CommitExecutionDenial::from(outcome.unwrap_err());
        let expected = if expired {
            Cause::DeadlineElapsed
        } else {
            Cause::Cancelled
        };
        assert_eq!(denial.kind, CommitExecutionDenialKind::Cause(expected));
        assert_eq!(began.get(), !expired);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
