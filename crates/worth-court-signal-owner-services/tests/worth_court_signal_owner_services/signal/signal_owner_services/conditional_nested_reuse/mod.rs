#[allow(unused_imports)]
use crate::signal_owner_services::ordinary_context::conditional_nested_reuse::*;
pub(crate) use worth_signal::facade::specialist::DefaultComparatorResolver;
pub(crate) use {
    worth_proof::ConditionalEvaluationSource, worth_proof::ConditionalSourceObservationOwner,
};
pub(crate) use {
    worth_signal::facade::branch::SignalConditionalServiceExecutionRequest as Request,
    worth_signal::facade::branch::SignalOwnerCancellationSource,
};
pub(crate) use {
    worth_signal::facade::Aspect, worth_signal::facade::AspectMask,
    worth_signal::facade::AspectVersion, worth_signal::facade::ConditionEvaluationContext,
    worth_signal::facade::DependencyEdge, worth_signal::facade::InstalledSignalConditionDecision,
    worth_signal::facade::InstalledSignalConditionIdentity,
    worth_signal::facade::InstalledSignalConditionResolver,
    worth_signal::facade::InstalledSignalConditionalContract,
    worth_signal::facade::NodeEvaluationResult, worth_signal::facade::SignalAspectLoweringOwner,
    worth_signal::facade::SignalConditionalArtifactReuse,
    worth_signal::facade::SignalConditionalCondition,
    worth_signal::facade::SignalConditionalContractDefinition,
    worth_signal::facade::SignalConditionalDecisionClass,
    worth_signal::facade::SignalConditionalVersionComparator, worth_signal::facade::SignalError,
    worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime,
};

#[test]
fn standalone_and_nested_routes_share_owner_slot_reuse_state() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut runtime, claimant, contract, source_owner) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let services = runtime.owner_component_services().unwrap();
    let mutation = services.mutation_port();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let warm_evaluation = service
        .admit_evaluation(
            &contract,
            worth_proof::ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("warm-source"),
            ),
        )
        .unwrap();
    let fresh_evaluation = service
        .admit_evaluation(
            &contract,
            worth_proof::ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("fresh-source"),
            ),
        )
        .unwrap();

    let mut standalone_computes = 0;
    let standalone = service
        .execute(
            request_execution,
            &warm_evaluation,
            worth_signal::facade::branch::SignalConditionalServiceExecutionRequest::new(1),
            &mut NoPredicate,
            &mut worth_signal::facade::specialist::DefaultComparatorResolver::default(),
            || {
                standalone_computes += 1;
                Ok(output(9))
            },
        )
        .unwrap();
    assert!(!standalone.slot_reused());
    assert_eq!(standalone_computes, 1);

    let cancellation = worth_signal::facade::branch::SignalOwnerCancellationSource::new();
    let mut warm_nested = None;
    let mut fresh_first = None;
    let mut fresh_second = None;
    let mut warm_nested_computes = 0;
    let mut fresh_first_computes = 0;
    let mut fresh_second_computes = 0;
    mutation
        .advance_exact(
            request_execution,
            &basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                warm_nested = Some(
                    service
                        .execute_within_transaction(
                            request_execution,
                            transaction,
                            &warm_evaluation,
                            worth_signal::facade::branch::SignalConditionalServiceExecutionRequest::new(2),
                            &mut NoPredicate,
                            &mut worth_signal::facade::specialist::DefaultComparatorResolver::default(),
                            || {
                                warm_nested_computes += 1;
                                Ok(output(10))
                            },
                        )
                        .unwrap(),
                );
                fresh_first = Some(
                    service
                        .execute_within_transaction(
                            request_execution,
                            transaction,
                            &fresh_evaluation,
                            worth_signal::facade::branch::SignalConditionalServiceExecutionRequest::new(1),
                            &mut NoPredicate,
                            &mut worth_signal::facade::specialist::DefaultComparatorResolver::default(),
                            || {
                                fresh_first_computes += 1;
                                Ok(output(11))
                            },
                        )
                        .unwrap(),
                );
                fresh_second = Some(
                    service
                        .execute_within_transaction(
                            request_execution,
                            transaction,
                            &fresh_evaluation,
                            worth_signal::facade::branch::SignalConditionalServiceExecutionRequest::new(2),
                            &mut NoPredicate,
                            &mut worth_signal::facade::specialist::DefaultComparatorResolver::default(),
                            || {
                                fresh_second_computes += 1;
                                Ok(output(12))
                            },
                        )
                        .unwrap(),
                );
                Ok(())
            },
        )
        .unwrap();

    let warm_nested = warm_nested.unwrap();
    let fresh_first = fresh_first.unwrap();
    let fresh_second = fresh_second.unwrap();
    assert!(warm_nested.slot_reused());
    assert!(!fresh_first.slot_reused());
    assert!(fresh_second.slot_reused());
    assert_eq!(warm_nested_computes, 0);
    assert_eq!(fresh_first_computes, 1);
    assert_eq!(fresh_second_computes, 0);
    assert_eq!(
        warm_nested.into_parts().0.unwrap().class(),
        worth_signal::facade::SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(
        fresh_first.into_parts().0.unwrap().class(),
        worth_signal::facade::SignalConditionalDecisionClass::ComputedChanged
    );
    assert_eq!(
        fresh_second.into_parts().0.unwrap().class(),
        worth_signal::facade::SignalConditionalDecisionClass::DependencyUnchanged
    );
}

#[path = "no_policy/mod.rs"]
pub(crate) mod no_policy;
