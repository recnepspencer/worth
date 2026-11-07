//! Original admission owns lowering storage until the last public template drops.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_runtime_bridge::facade::*;
mod source;

// Observe the retained owner footprint, not a newly hard-coded allocation peak.
fn retained_template_bytes() -> u64 {
    let budget = SerialMemoryBudget::new(None);
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let runtime = source::runtime();
    let basis = source::admit(&runtime, source::planned(&runtime, execution), execution).unwrap();
    let template = basis.request().lowered().clone();
    drop(basis);
    drop(runtime);
    let bytes = budget.charged_memory_bytes();
    assert!(bytes > 0);
    drop(template);
    assert_eq!(budget.charged_memory_bytes(), 0);
    bytes
}

#[test]
fn original_lowering_charge_lives_until_the_last_shared_template_drops() {
    let bytes = retained_template_bytes();
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let runtime = source::runtime();
    let basis = source::admit(&runtime, source::planned(&runtime, execution), execution).unwrap();
    let template = basis.request().lowered().clone();
    let first = template.clone();
    let second = first.clone();
    drop(basis);
    drop(runtime);
    assert_eq!(budget.charged_memory_bytes(), bytes);
    cancellation.cancel();
    drop(serial);
    drop(template);
    drop(first);
    assert_eq!(budget.charged_memory_bytes(), bytes);
    drop(second);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn original_budget_refuses_lowering_backing_without_retaining_a_charge() {
    let budget = SerialMemoryBudget::new(Some(retained_template_bytes() - 1));
    let serial =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let runtime = source::runtime();
    let denial =
        source::admit(&runtime, source::planned(&runtime, execution), execution).unwrap_err();
    assert!(matches!(
        denial.kind(),
        BridgeExecutionBasisDenialKind::ExecutionDenied(BridgeExecutionDenial::MemoryExhausted(_))
    ));
    assert_eq!(denial.counters().truth_materialization_count(), 0);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn cancelled_original_request_refuses_lowering_backing() {
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let serial = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&serial);
    let runtime = source::runtime();
    let planned = source::planned(&runtime, execution);
    cancellation.cancel();
    let denial = source::admit(&runtime, planned, execution).unwrap_err();
    assert_eq!(
        denial.kind(),
        BridgeExecutionBasisDenialKind::ExecutionDenied(BridgeExecutionDenial::Cancelled)
    );
    assert_eq!(denial.counters().truth_materialization_count(), 0);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
