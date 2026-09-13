use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};
use worth_signal::facade::branch::{
    SignalConditionalServiceExecutionRequest as Request, SignalOwnerCancellationSource,
};
use worth_signal::facade::specialist::DefaultComparatorResolver;
use worth_signal::facade::{
    Aspect, AspectMask, AspectVersion, ConditionEvaluationContext, DependencyEdge,
    InstalledSignalConditionDecision, InstalledSignalConditionIdentity,
    InstalledSignalConditionResolver, InstalledSignalConditionalContract, NodeEvaluationResult,
    SignalAspectLoweringOwner, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator, SignalError, SignalGraph, SignalRuntime,
};

struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &InstalledSignalConditionIdentity,
        _: &ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always does not contact a predicate provider")
    }
}

fn runtime_with_contract() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalAspectLoweringOwner,
    InstalledSignalConditionalContract,
    ConditionalSourceObservationOwner,
) {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let dependency = graph.create_node();
    let rollback_target = graph.create_node();
    graph
        .set_dependencies(
            rollback_target,
            [DependencyEdge::new(dependency, Aspect::new(1))],
        )
        .unwrap();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node must admit")
    };
    let contract = graph
        .install_conditional_contract(
            &claimant,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::Always,
                dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
                trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    (
        SignalRuntime::build_for::<()>(graph),
        claimant,
        contract,
        ConditionalSourceObservationOwner::fresh(),
    )
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn standalone_and_nested_routes_share_owner_slot_reuse_state() {
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
            ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("warm-source"),
            ),
        )
        .unwrap();
    let fresh_evaluation = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("fresh-source"),
            ),
        )
        .unwrap();

    let mut standalone_computes = 0;
    let standalone = service
        .execute(
            &warm_evaluation,
            Request::new(1),
            &mut NoPredicate,
            &mut DefaultComparatorResolver::default(),
            || {
                standalone_computes += 1;
                Ok(output(9))
            },
        )
        .unwrap();
    assert!(!standalone.slot_reused());
    assert_eq!(standalone_computes, 1);

    let cancellation = SignalOwnerCancellationSource::new();
    let mut warm_nested = None;
    let mut fresh_first = None;
    let mut fresh_second = None;
    let mut warm_nested_computes = 0;
    let mut fresh_first_computes = 0;
    let mut fresh_second_computes = 0;
    mutation
        .advance_exact(&basis, &mut (), &cancellation.token(), |transaction| {
            warm_nested = Some(
                service
                    .execute_within_transaction(
                        transaction,
                        &warm_evaluation,
                        Request::new(2),
                        &mut NoPredicate,
                        &mut DefaultComparatorResolver::default(),
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
                        transaction,
                        &fresh_evaluation,
                        Request::new(1),
                        &mut NoPredicate,
                        &mut DefaultComparatorResolver::default(),
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
                        transaction,
                        &fresh_evaluation,
                        Request::new(2),
                        &mut NoPredicate,
                        &mut DefaultComparatorResolver::default(),
                        || {
                            fresh_second_computes += 1;
                            Ok(output(12))
                        },
                    )
                    .unwrap(),
            );
            Ok(())
        })
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
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(
        fresh_first.into_parts().0.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert_eq!(
        fresh_second.into_parts().0.unwrap().class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
}
