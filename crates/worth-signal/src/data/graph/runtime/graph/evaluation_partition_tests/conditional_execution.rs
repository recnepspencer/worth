use crate::data::aspect::{Aspect, AspectMask, AspectVersion, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass,
    SignalConditionalExecutionRequest, SignalConditionalVersionComparator,
};
use crate::data::error::SignalError;
use crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
mod draft;
mod finalization;
mod topology;
mod unwind;
mod upstream;
mod waiters;
mod work;

struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always does not contact a predicate provider")
    }
}

fn installed() -> (SignalGraph, InstalledSignalConditionalContract) {
    installed_with(
        SignalConditionalCondition::Always,
        SignalConditionalArtifactReuse::NotReusable,
    )
}

fn installed_with(
    condition: SignalConditionalCondition,
    artifact_reuse: SignalConditionalArtifactReuse,
) -> (SignalGraph, InstalledSignalConditionalContract) {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node must admit")
    };
    let contract = graph
        .install_conditional_contract(
            &owner,
            capability,
            SignalConditionalContractDefinition {
                condition,
                dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
                trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse,
            },
        )
        .unwrap();
    (graph, contract)
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn named_execution_preserves_warm_conditional_state_and_closes_observation() {
    let (mut graph, contract) = installed();
    let mut b = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut a = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut computes = 0;
    // These are storage-kernel partitions, not source-admission fixtures. Equal
    // reporting labels deliberately cannot explain the isolated results.
    for (partition, value, expected) in [
        (&mut b, 4, SignalConditionalDecisionClass::ComputedChanged),
        (&mut a, 6, SignalConditionalDecisionClass::ComputedChanged),
    ] {
        let completion = partition
            .execute_conditional(
                &mut graph,
                SignalConditionalExecutionRequest::new(&contract, "storage", "execution", 1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(value))
                },
            )
            .unwrap();
        let (decision, observation, _rejected) = completion.into_parts();
        assert_eq!(decision.unwrap().class(), expected);
        assert!(observation.unwrap().is_some());
    }
    let (decision, observation, _rejected) = b
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "execution", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(99))
            },
        )
        .unwrap()
        .into_parts();
    let decision = decision.unwrap();
    assert_eq!(
        decision.class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(decision.counters().compute_contacts, 0);
    assert_eq!(computes, 2);
    assert!(observation.unwrap().is_none());
    b.execute(&mut graph, |graph| {
        assert_eq!(
            graph
                .node_version_for_scope(contract.node(), Aspect::new(0), None)
                .unwrap(),
            4
        );
        assert_eq!(graph.observation_session_active_generation(), 0);
    })
    .unwrap();
    assert_eq!(
        graph
            .node_version_for_scope(contract.node(), Aspect::new(0), None)
            .unwrap(),
        0
    );
}

#[test]
fn compute_failure_closes_observation_and_allows_a_fresh_operation() {
    let (mut graph, contract) = installed();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let (decision, observation, _rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "failed", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Err(SignalError::invalid_input("provider failure")),
        )
        .unwrap()
        .into_parts();
    let failure = match decision {
        Err(failure) => failure,
        Ok(_) => panic!("compute must fail"),
    };
    assert_eq!(failure.counters().compute_contacts, 1);
    assert!(observation.unwrap().is_none());
    let (decision, observation, _rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "retry", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(8)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.unwrap().is_some());
}

#[test]
fn retained_conditional_executes_old_definition_after_current_membership_grows() {
    let (mut graph, contract) = installed();
    let original = graph.node_eval_config(contract.node()).unwrap().clone();
    let mut retained = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let old_pages = graph.arena.definitions.page_identities();
    retained
        .execute(&mut graph, |selected| {
            assert_eq!(selected.arena.definitions.page_identities(), old_pages);
        })
        .unwrap();

    let added = graph.create_node();
    let changed = crate::data::node::NodeEvaluationConfig {
        condition: crate::data::node::EvaluationCondition::OnDemand,
        ..original.clone()
    };
    graph
        .get_entry_mut(contract.node())
        .unwrap()
        .set_eval_config(changed.clone());
    let current_ledger = graph.pending_branch_mutation_records();
    let current_pages = graph.arena.definitions.page_identities();

    let (decision, observation, _rejected) = retained
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "retained", "old-definition", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(4)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.unwrap().is_some());
    retained
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected.node_eval_config(contract.node()).unwrap(),
                &original
            );
            assert!(selected.get_entry(added).is_err());
            assert_eq!(selected.arena.definitions.page_identities(), old_pages);
        })
        .unwrap();
    assert_eq!(graph.node_eval_config(contract.node()).unwrap(), &changed);
    assert!(graph.get_entry(added).is_ok());
    assert_eq!(graph.arena.definitions.page_identities(), current_pages);
    assert_eq!(graph.pending_branch_mutation_records(), current_ledger);
}
