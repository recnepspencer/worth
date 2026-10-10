use worth_proof::TransitionOutcome;

use crate::data::aspect::{Aspect, AspectMask, SignalAspectLoweringOwner};
use crate::data::graph::SignalGraph;

use super::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalVersionComparator,
    SignalDeltaThresholdContract, SignalThresholdBoundary, SignalThresholdComparisonDomain,
    SignalThresholdValueFamily,
};

struct UnexpectedInstalledConditionResolver;

impl InstalledSignalConditionResolver for UnexpectedInstalledConditionResolver {
    fn resolve(
        &mut self,
        _identity: &crate::data::node::InstalledSignalConditionIdentity,
        _context: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, crate::data::error::SignalError> {
        panic!("aspect-filter contract must not consult an installed predicate")
    }
}

#[test]
fn reinstalling_one_graph_node_preserves_role_bound_provider_identities() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let definition = SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::RuntimePredicate,
        dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
        trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
        dependency_comparator: SignalConditionalVersionComparator::RuntimeResolved,
        output_comparator: SignalConditionalVersionComparator::RuntimeResolved,
        artifact_reuse: SignalConditionalArtifactReuse::RuntimeResolved,
    };

    let TransitionOutcome::Success(first_capability) = graph.admit_installed_node(node) else {
        panic!("fresh node should admit")
    };
    let first = graph
        .install_conditional_contract(&owner, first_capability, definition.clone())
        .unwrap();
    let TransitionOutcome::Success(second_capability) = graph.admit_installed_node(node) else {
        panic!("installed node should remain current")
    };
    let second = graph
        .install_conditional_contract(&owner, second_capability, definition)
        .unwrap();

    assert!(first.compare_semantic_continuity(&second).is_ok());
    assert!(first.compare_execution_affinity(&second).is_ok());
    assert!(matches!(
        graph.node_eval_config(node).unwrap().comparator,
        Some(crate::data::comparator::VersionComparatorPolicy::Installed { ref identity })
            if identity.role() == crate::data::comparator::InstalledSignalComparatorRole::DependencyVersion
    ));
    assert!(matches!(
        graph.node_eval_config(node).unwrap().output_equivalence,
        crate::data::output_equivalence::OutputEquivalencePolicy::Installed { .. }
    ));
}

#[test]
fn unitful_threshold_meaning_survives_signal_installation_without_numeric_flattening() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let TransitionOutcome::Success(node_capability) = graph.admit_installed_node(node) else {
        panic!("fresh node should admit")
    };
    let threshold = SignalDeltaThresholdContract::new(
        worth_foundational::facade::AspectValue::Float64(
            worth_foundational::facade::CanonicalF64::from_f64(0.01),
        ),
        "worth.tests.units.millimeters",
        SignalThresholdValueFamily::Float64,
        SignalThresholdComparisonDomain::AbsoluteDifference,
        SignalThresholdBoundary::Inclusive,
    );
    let contract = graph
        .install_conditional_contract(
            &owner,
            node_capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::DeltaThreshold(threshold.clone()),
                dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
                trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();

    assert_eq!(
        contract.semantic_condition(),
        &SignalConditionalCondition::DeltaThreshold(threshold)
    );
    assert!(matches!(
        contract.condition(),
        crate::data::node::EvaluationCondition::Installed(_)
    ));
}

#[test]
fn installed_condition_defers_pending_dependency_before_filtering() {
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution_request = worth_execution::ExecutionRequest::serial(&serial);

    use crate::data::comparator::DefaultComparatorPolicyResolver;
    use crate::data::dependency::DependencyEdge;
    use crate::facade::mark_dirty;
    use crate::tests::support::{evaluate, version_ab};

    let dependency_aspect = Aspect::new(1);
    let filter_aspect = Aspect::new(0);
    let mut graph = SignalGraph::new();
    let source = graph.node().produces_aspects(dependency_aspect).build();
    let node = graph.node().produces_aspects(filter_aspect).build();
    graph
        .set_dependencies(node, [DependencyEdge::new(source, dependency_aspect)])
        .unwrap();
    evaluate(&mut graph, source, &mut |_node, _graph| {
        Ok(version_ab(0, 1))
    })
    .unwrap();
    evaluate(&mut graph, node, &mut |_node, _graph| Ok(version_ab(1, 0))).unwrap();

    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node) else {
        panic!("current node should admit")
    };
    let contract = graph
        .install_conditional_contract(
            &owner,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::AspectFilter(AspectMask::from_aspect(
                    filter_aspect,
                )),
                dependency_aspects: AspectMask::from_aspect(dependency_aspect),
                trigger_aspects: AspectMask::from_aspect(filter_aspect),
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    mark_dirty(&mut graph, source, dependency_aspect).unwrap();
    let mut compute_contacts = 0;
    let evidence = graph
        .execute_installed_conditional(
            execution_request,
            super::SignalConditionalExecutionRequest::new(&contract, "snapshot", "attempt", 1),
            &mut UnexpectedInstalledConditionResolver,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                compute_contacts += 1;
                Ok(crate::data::output::NodeEvaluationResult::from_version(
                    version_ab(2, 0),
                ))
            },
        )
        .unwrap();

    assert_eq!(
        evidence.class(),
        super::SignalConditionalDecisionClass::DeferredByCondition
    );
    assert_eq!(compute_contacts, 0);
    assert_eq!(
        graph.get_state(node).unwrap(),
        crate::data::node::NodeState::MaybeStale
    );
}
