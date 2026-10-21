//! Genuine service slots, original results and finite host refusal.
use super::*;
use std::sync::Arc;
use worth_signal::facade::branch::SignalConditionalServiceExecutionDenial as Denial;
use worth_signal::facade::runtime::SignalConditionalEvaluationBudget;
use worth_signal::facade::SignalRuntimePolicy;

#[test]
fn absent_owner_policy_keeps_192_original_admissions_and_results_until_release() {
    let (mut runtime, claimant, contract, source_owner) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_component_services().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let mut admissions = Vec::new();
    let mut results = Vec::new();
    for ordinal in 0..192 {
        let admitted = Arc::new(
            service
                .admit_evaluation(
                    &contract,
                    ConditionalEvaluationSource::AdmittedRelationalSource(
                        source_owner.admit(format!("original-{ordinal}")),
                    ),
                )
                .unwrap(),
        );
        let result = service
            .execute(
                execution,
                &admitted,
                Request::new(ordinal),
                &mut NoPredicate,
                &mut DefaultComparatorResolver::default(),
                || Ok(output(7)),
            )
            .unwrap();
        assert!(!result.slot_reused());
        results.push(result);
        admissions.push(admitted);
    }
    let held = service.retention_observation().unwrap();
    assert_eq!(held.retained_slots(), admissions.len());
    assert!(held.retained_bytes() > 0);
    let clone = Arc::clone(&admissions[0]);
    assert_eq!(service.retention_observation().unwrap(), held);
    drop(admissions);
    assert!(service.retention_observation().unwrap().retained_slots() >= 1);
    assert!(results.iter().all(|result| !result.slot_reused()));
    drop(results);
    drop(clone);
    assert_eq!(service.retention_observation().unwrap().retained_slots(), 0);
    drop(runtime);
    assert!(
        service.retention_observation().is_err(),
        "close fences new owner work"
    );
}

#[test]
fn explicit_single_slot_policy_refuses_before_second_admission_and_refunds() {
    let (mut runtime, claimant, contract, source_owner) = runtime_with_contract();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: Some(1),
                maximum_retained_bytes: Some(512 * 1024 * 1024),
                maximum_attempt_visits: None,
            },
        ),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_component_services().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let first = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("first")),
        )
        .unwrap();
    let held = service.retention_observation().unwrap();
    assert!(matches!(
        service.admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("refused"))
        ),
        Err(Denial::AdmissionCapacityExhausted)
    ));
    assert_eq!(service.retention_observation().unwrap(), held);
    drop(first);
    assert_eq!(service.retention_observation().unwrap().retained_slots(), 0);
    let next = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("next")),
        )
        .unwrap();
    drop(next);
    assert_eq!(service.retention_observation().unwrap().retained_slots(), 0);
}

#[test]
fn explicit_byte_policy_refuses_basis_capture_before_service_admission() {
    let (mut runtime, claimant, _contract, source_owner) = runtime_with_contract();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: None,
                maximum_retained_bytes: Some(1),
                maximum_attempt_visits: None,
            },
        ),
    );
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_component_services().unwrap();
    assert!(matches!(
        runtime.issue_conditional_execution_service(&basis, &claimant, &source_owner.authority()),
        Err(worth_signal::facade::branch::SignalConditionalServiceIssuanceDenial::CaptureCapacityExhausted)
    ));
}
