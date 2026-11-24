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
    let authority = crate::execution::test_authority::test_execution_authority();
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
    let scan =
        crate::execution::ordered_scan::admit_ordered_scan(vec![(PartitionIdentity::new(1), ())])
            .unwrap();
    let required = scan.declared_memory_requirement_for_lease::<(), PreparedArtifact<Vec<u8>>, crate::execution::read_only_packets::PacketBudgetDenial>(
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
    crate::execution::test_authority::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, work),
            ),
            deadline: deadline
                .map(|instant| worth_execution::MonotonicClock::system().deadline_at(instant)),
            cancellation,
        })
        .unwrap()
}

#[test]
fn tracked_artifacts_use_the_original_request_and_shared_remaining_work() {
    let lease = tracked_lease(Some(3), CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let work = crate::execution::request_work::RequestWorkBudget::new();
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
    let denial =
        crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial::from(
            second.unwrap_err(),
        );
    assert_eq!(
        denial.kind,
        crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(
            crate::execution::denial_cause::RelationalExecutionDenialCause::WorkExhausted
        )
    );
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn tracked_artifacts_keep_original_cancellation_deadline_and_release_scope_memory() {
    use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
    use crate::transactions::data::outcomes::transaction_commit_error::{
        CommitExecutionDenial, CommitExecutionDenialKind,
    };
    use std::{
        cell::Cell,
        time::{Duration, Instant},
    };
    for expired in [false, true] {
        let cancellation = worth_execution::CancellationSource::new();
        let deadline = expired.then(|| Instant::now() - Duration::from_secs(1));
        let lease = tracked_lease(None, cancellation.token(), deadline);
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        let work = crate::execution::request_work::RequestWorkBudget::new();
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

/// The retained bill of a public immutable byte owner holding `bytes`,
/// measured after its original controller scope is gone: payload plus control.
pub(crate) fn retained_byte_owner_bill(bytes: &[u8]) -> u64 {
    let budget = worth_execution::SerialMemoryBudget::new(Some(
        (bytes.len() as u64).checked_add(64 * 1024).unwrap(),
    ));
    let host = crate::execution::test_policy::serial_request_with_memory(budget.clone());
    let request = worth_execution::ExecutionRequest::serial(&host);
    let mut building =
        worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(request, bytes.len())
            .unwrap();
    building.extend_from_slice(bytes).unwrap();
    let owned = building.seal_immutable_bytes().unwrap();
    assert_eq!(owned.charged_payload_bytes(), Some(bytes.len() as u64));
    let total = budget.charged_memory_bytes();
    assert!(total > bytes.len() as u64);
    drop(owned);
    assert_eq!(budget.charged_memory_bytes(), 0);
    total
}
