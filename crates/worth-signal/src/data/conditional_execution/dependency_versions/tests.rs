mod work_tests;
use worth_proof::TransitionOutcome;

use crate::data::aspect::{Aspect, AspectMask, AspectVersion, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass as Class,
    SignalConditionalDecisionEvidence, SignalConditionalExecutionRequest,
    SignalConditionalVersionComparator,
};
use crate::data::error::SignalError;
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;

struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always must not contact a predicate provider")
    }
}

fn install(
    graph: &mut SignalGraph,
    owner: &SignalAspectLoweringOwner,
    node: NodeId,
    aspects: AspectMask,
) -> InstalledSignalConditionalContract {
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node) else {
        panic!("live node must admit")
    };
    graph
        .install_conditional_contract(
            owner,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::Always,
                dependency_aspects: aspects,
                trigger_aspects: aspects,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap()
}

fn read_clean(
    graph: &mut SignalGraph,
    contract: &InstalledSignalConditionalContract,
) -> SignalConditionalDecisionEvidence {
    graph
        .execute_installed_conditional(
            SignalConditionalExecutionRequest::new(contract, "source", "read-clean", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || panic!("clean node must not compute"),
        )
        .unwrap()
}

#[test]
fn conditional_reinstallation_compares_named_aspects_and_preserves_retained_warm_slots() {
    let one = AspectMask::from(Aspect::new(1));
    let two = AspectMask::from(Aspect::new(2));
    for (before, after) in [(one, one | two), (one | two, two), (one, two), (one, one)] {
        let mut graph = SignalGraph::new();
        let node = graph.node().build();
        let owner = SignalAspectLoweringOwner::fresh();
        graph.claim_aspect_lowering_owner(&owner).unwrap();
        let original = install(&mut graph, &owner, node, before);
        let first = graph
            .execute_installed_conditional(
                SignalConditionalExecutionRequest::new(&original, "source", "initial", 1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    Ok(NodeEvaluationResult::from_version(
                        AspectVersion::from_updates([(Aspect::new(0), 7)]),
                    ))
                },
            )
            .unwrap();
        assert_eq!(first.class(), Class::ComputedChanged);
        assert_eq!(first.counters().compute_contacts, 1);
        // Both dependency coordinates have the same numeric value. Positional
        // comparison would falsely reuse them when the mask changes.
        for aspect in [Aspect::new(1), Aspect::new(2)] {
            assert_eq!(graph.node_version_for_scope(node, aspect, None).unwrap(), 0);
        }
        assert_eq!(
            read_clean(&mut graph, &original).class(),
            Class::DependencyUnchanged
        );
        let mut retained = SignalEvaluationPartition::retain_basis_storage(&mut graph);
        let replacement = install(&mut graph, &owner, node, after);
        let changed = read_clean(&mut graph, &replacement);
        assert_eq!(
            changed.class(),
            if before == after {
                Class::DependencyUnchanged
            } else {
                Class::SuppressedBeforeCompute
            }
        );
        assert_eq!(
            changed.counters().condition_checks,
            usize::from(before != after)
        );
        assert_eq!(changed.counters().compute_contacts, 0);
        assert_eq!(
            read_clean(&mut graph, &replacement).class(),
            Class::DependencyUnchanged
        );
        // This is retained storage isolation, not service/source admission.
        let (decision, observation, rejected) = retained
            .execute_conditional(
                &mut graph,
                SignalConditionalExecutionRequest::new(&original, "source", "retained", 3),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || panic!("retained warm node must not compute"),
            )
            .unwrap()
            .into_parts();
        assert!(rejected.is_none());
        assert_eq!(decision.unwrap().class(), Class::DependencyUnchanged);
        assert!(observation.unwrap().is_none());
        assert_eq!(
            read_clean(&mut graph, &replacement).class(),
            Class::DependencyUnchanged
        );
    }
}

#[test]
fn conditional_version_observation_has_fixed_inline_retention_and_bounded_measurement() {
    use crate::data::retained_storage::{
        RetainedStorageCharge, RetainedStorageMeasurement, RetainedStoragePreparation,
        RetainedStoragePreparationDenial,
    };
    for mask in [AspectMask::EMPTY, AspectMask::ALL] {
        let observation =
            super::SignalConditionalVersionObservation::new(mask, AspectVersion::zero());
        let mut work = RetainedStoragePreparation::new(2);
        assert_eq!(
            observation.retained_heap_charge(&mut work),
            Ok(RetainedStorageCharge::ZERO)
        );
        assert_eq!(work.visits(), 2);
        assert!(matches!(
            observation.retained_heap_charge(&mut RetainedStoragePreparation::new(1)),
            Err(RetainedStoragePreparationDenial::WorkExhausted { .. })
        ));
    }
}
