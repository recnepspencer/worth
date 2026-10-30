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

pub(crate) struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &InstalledSignalConditionIdentity,
        _: &ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always does not contact a predicate provider")
    }
}

pub(crate) fn runtime_with_contract() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalAspectLoweringOwner,
    InstalledSignalConditionalContract,
    ConditionalSourceObservationOwner,
) {
    let mut graph = worth_signal::facade::SignalGraph::new();
    let claimant = worth_signal::facade::SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let dependency = graph.create_node();
    let rollback_target = graph.create_node();
    graph
        .set_dependencies(
            rollback_target,
            [worth_signal::facade::DependencyEdge::new(
                dependency,
                worth_signal::facade::Aspect::new(1),
            )],
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
                condition: worth_signal::facade::SignalConditionalCondition::Always,
                dependency_aspects: worth_signal::facade::AspectMask::from_aspect(
                    worth_signal::facade::Aspect::new(1),
                ),
                trigger_aspects: worth_signal::facade::AspectMask::from_aspect(
                    worth_signal::facade::Aspect::new(1),
                ),
                dependency_comparator:
                    worth_signal::facade::SignalConditionalVersionComparator::Exact,
                output_comparator: worth_signal::facade::SignalConditionalVersionComparator::Exact,
                artifact_reuse: worth_signal::facade::SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    (
        worth_signal::facade::SignalRuntime::build_for::<()>(graph),
        claimant,
        contract,
        worth_proof::ConditionalSourceObservationOwner::fresh(),
    )
}

pub(crate) fn output(value: u64) -> NodeEvaluationResult {
    worth_signal::facade::NodeEvaluationResult::from_version(
        worth_signal::facade::AspectVersion::from_updates([(
            worth_signal::facade::Aspect::new(0),
            value,
        )]),
    )
}
