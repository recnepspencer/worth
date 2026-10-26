//! Real original-request accounting and control, independent of graph semantics.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, LeaseRequest, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[derive(Debug)]
struct Payload(Vec<u8>);
impl ChargedBytes for Payload {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}
fn policy(memory: u64, work: u64) -> ExecutionRequestPolicy {
    ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), Some(memory), Some(work)),
    )
}
fn cause(error: TransactionCommitError) -> Cause {
    let TransactionCommitError::Execution { denial, .. } = error else {
        panic!("execution denial required: {error:?}")
    };
    let crate::transactions::data::CommitExecutionDenialKind::Cause(cause) = denial.kind;
    cause
}

#[test]
fn dependent_stages_spend_one_request_work_account_and_release_result_custody() {
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: policy(256 * 1024, 100),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let account = crate::execution::RequestWorkBudget::new();
    let before = lease.charged_memory_bytes();
    let (value, retention) = run(&execution, Some(&account), |work| {
        work.retain(64)?;
        work.checkpoint(60)?;
        Ok(Payload(vec![0; 64]))
    })
    .unwrap();
    assert!(retention.is_some());
    assert!(lease.charged_memory_bytes() >= before + 64);
    let refused = run(&execution, Some(&account), |work| {
        work.checkpoint(40)?;
        Ok(Payload(Vec::new()))
    });
    let Err(error) = refused else {
        panic!("a second stage cannot reset the request's remaining39 units")
    };
    assert_eq!(cause(error), Cause::WorkExhausted);
    drop(value);
    drop(retention);
    assert_eq!(lease.charged_memory_bytes(), before);
}

#[test]
fn before_growth_memory_refusal_preserves_the_original_policy_cause() {
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: policy(64 * 1024, 1000),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let began = Cell::new(false);
    let allocated = Cell::new(false);
    let before = lease.charged_memory_bytes();
    let result = run(&execution, None, |work| {
        began.set(true);
        work.retain(1024 * 1024)?;
        allocated.set(true);
        Ok(Payload(vec![0; 1024 * 1024]))
    });
    let Err(error) = result else {
        panic!("the original64 KiB policy must refuse before allocation")
    };
    assert!(began.get());
    assert!(!allocated.get());
    assert!(matches!(cause(error), Cause::PolicyMemoryExhausted { .. }));
    assert_eq!(lease.charged_memory_bytes(), before);
}

#[test]
fn cancellation_inside_preparation_drops_prefix_and_original_memory_ticket() {
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_policy(&policy(256 * 1024, 1000), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&host);
    let result = run(&execution, None, |work| {
        work.retain(64)?;
        let _prefix = vec![0u8; 64];
        cancellation.cancel();
        work.checkpoint(1)?;
        Ok(Payload(Vec::new()))
    });
    let Err(error) = result else {
        panic!("no prefix escapes cancellation")
    };
    assert_eq!(cause(error), Cause::Cancelled);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn unaccounted_serial_backing_cannot_hide_its_real_accounted_ancestor() {
    let worker_only = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        NonZeroUsize::new(1).unwrap(),
    );
    let child = SerialRequest::from_policy(&worker_only, CancellationToken::new(), None);
    let original =
        SerialRequest::from_policy(&policy(64 * 1024, 1000), CancellationToken::new(), None);
    let seen = Cell::new(false);
    let captured = Cell::new(None);
    let outer = ExecutionRequest::serial(&original).run_native_with_checkpoint(|_| {
        let result = run(&ExecutionRequest::serial(&child), None, |work| {
            seen.set(work.context.has_accounting());
            work.retain(1024 * 1024)?;
            Ok(Payload(Vec::new()))
        });
        captured.set(result.err().map(cause));
    });
    assert!(matches!(
        outer,
        Ok(((), _))
            | Err(worth_execution::WorkCeilingDenial::Stopped(
                worth_execution::MapKernelStop::NestedStopped
            ))
    ));
    assert!(
        matches!(captured.get(), Some(Cause::PolicyMemoryExhausted { .. })),
        "original refusal: {:?}; accounting reached: {}",
        captured.get(),
        seen.get()
    );
    assert!(seen.get());
    assert_eq!(original.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn native_custom_read_debits_rule_once_and_original_enclosing_report_once() {
    use crate::validation::CustomInvariantWorkMeter;
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: policy(256 * 1024, 100),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let account = crate::execution::RequestWorkBudget::new();
    let rule = CustomInvariantWorkMeter::new(std::num::NonZeroU64::new(60));
    let outer = run(&execution, Some(&account), |work| {
        work.checkpoint(7)?;
        let mut allowance = |units| {
            rule.try_debit_native_work(units)
                .then_some(())
                .ok_or(Cause::WorkExhausted)
        };
        let (_, retention) = run_in_custom_read(&execution, &mut allowance, |nested| {
            nested.checkpoint(60)?;
            Ok(Payload(Vec::new()))
        })
        .expect("nested read borrows the active original frame, never its locked account");
        drop(retention);
        work.checkpoint(11)?;
        Ok(Payload(Vec::new()))
    })
    .unwrap();
    drop(outer);
    assert_eq!(rule.charged_work(), 60);
    drop(rule);
    let remainder = run(&execution, Some(&account), |work| {
        work.checkpoint(22)?;
        Ok(Payload(Vec::new()))
    })
    .expect("78 original units leave exactly22, not a twice-billed read");
    drop(remainder);
    let exhausted = run(&execution, Some(&account), |work| {
        work.checkpoint(1)?;
        Ok(Payload(Vec::new()))
    });
    assert_eq!(cause(exhausted.unwrap_err()), Cause::WorkExhausted);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
