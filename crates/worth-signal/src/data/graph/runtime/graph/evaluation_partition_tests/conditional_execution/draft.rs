use super::*;
use crate::data::comparator::{
    ComparatorPolicyResolver, InstalledSignalComparatorIdentity, InstalledSignalComparatorRole,
    VersionComparatorPolicy, VersionComparatorResolver,
};
use crate::data::graph::storage::evaluation_partition::SignalPartitionConditionalUnwindReason;
use crate::facade::SignalRuntimePolicy;
use std::panic::{catch_unwind, AssertUnwindSafe};

struct DeclineReuse;

impl VersionComparatorResolver for DeclineReuse {
    fn resolve(&mut self, _: &str, _: Aspect, _: u64, _: u64) -> Result<bool, SignalError> {
        panic!("fixture installs an exact comparator identity")
    }

    fn resolve_installed(
        &mut self,
        identity: &InstalledSignalComparatorIdentity,
        _: Aspect,
        _: u64,
        _: u64,
    ) -> Result<bool, SignalError> {
        assert_eq!(
            identity.role(),
            InstalledSignalComparatorRole::ArtifactReuse
        );
        Err(SignalError::invalid_input(
            "reuse declined after application",
        ))
    }
}

impl ComparatorPolicyResolver for DeclineReuse {
    fn policy_for_node(
        &self,
        _: crate::data::handle::NodeId,
        node_override: Option<&VersionComparatorPolicy>,
    ) -> VersionComparatorPolicy {
        node_override
            .cloned()
            .unwrap_or(VersionComparatorPolicy::Exact)
    }
}

fn snapshot(
    partition: &mut SignalEvaluationPartition,
    graph: &mut SignalGraph,
    contract: &InstalledSignalConditionalContract,
) -> (u64, u64, serde_json::Value, Option<Vec<String>>) {
    partition
        .execute(graph, |selected| {
            (
                selected
                    .node_version_for_scope(contract.node(), Aspect::new(0), None)
                    .unwrap(),
                selected.cause_sets.output_commit_ordinal_for_test(),
                serde_json::to_value(selected.diagnostics_state()).unwrap(),
                selected
                    .node_retained_diagnostic_artifact(contract.node())
                    .unwrap()
                    .map(|artifact| artifact.labels.clone()),
            )
        })
        .unwrap()
}

fn assert_restored_contents(
    before: &(u64, u64, serde_json::Value, Option<Vec<String>>),
    after: (u64, u64, serde_json::Value, Option<Vec<String>>),
) {
    assert_eq!((after.0, after.1), (before.0, before.1));
    assert_eq!(after.3, before.3);
    let mut before = before.2.clone();
    let mut after = after.2;
    for allocator in [
        "next_replay_cursor",
        "next_snapshot_id",
        "next_branch_id",
        "next_lineage_artifact_id",
        "next_lineage_sequence",
    ] {
        let earlier = before
            .as_object_mut()
            .unwrap()
            .remove(allocator)
            .unwrap()
            .as_u64()
            .unwrap();
        let later = after
            .as_object_mut()
            .unwrap()
            .remove(allocator)
            .unwrap()
            .as_u64()
            .unwrap();
        assert!(later >= earlier, "issuance never rolls back: {allocator}");
    }
    assert_eq!(after, before);
}

#[test]
fn returned_finalization_failure_retains_rejected_diagnostics_without_installing_them() {
    let (mut graph, contract) = installed_with(
        SignalConditionalCondition::Always,
        SignalConditionalArtifactReuse::RuntimeResolved,
    );
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let (decision, observation, rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "warm", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(4).with_label("accepted output")),
        )
        .unwrap()
        .into_parts();
    decision.unwrap();
    assert!(observation.unwrap().is_some());
    assert!(rejected.is_none());
    let before = snapshot(&mut partition, &mut graph, &contract);

    let (decision, observation, rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "declined", 2)
                .force_on_demand(),
            &mut NoPredicate,
            &mut DeclineReuse,
            || Ok(output(4).with_label("rejected output")),
        )
        .unwrap()
        .into_parts();
    let failure = decision
        .err()
        .expect("reuse must fail after computed application");
    assert_eq!(failure.counters().compute_contacts, 1);
    assert_eq!(failure.counters().application_contacts, 1);
    assert_eq!(failure.counters().semantic_classifications, 1);
    assert_eq!(failure.counters().reuse_checks, 1);
    assert_eq!(
        failure.into_error(),
        SignalError::invalid_input("reuse declined after application")
    );
    observation.unwrap();
    let rejected = rejected.expect("returned failure owns rejected diagnostic state");
    assert_eq!(
        rejected
            .retained_artifact_for_test(contract.node())
            .unwrap()
            .labels,
        ["rejected output"]
    );
    assert_restored_contents(&before, snapshot(&mut partition, &mut graph, &contract));

    let (decision, observation, retry_rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "retry", 3)
                .force_on_demand(),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(8).with_label("retry output")),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(
        observation.unwrap().is_none(),
        "forced recomputation has no admitted invalidation work to receipt"
    );
    assert!(retry_rejected.is_none());
    assert_eq!(snapshot(&mut partition, &mut graph, &contract).0, 8);
    assert_eq!(
        rejected
            .retained_artifact_for_test(contract.node())
            .unwrap()
            .labels,
        ["rejected output"]
    );
}

#[test]
fn observation_unwind_rejects_computed_output_but_retains_performed_decision_and_draft() {
    let (mut graph, contract) = installed_with(
        SignalConditionalCondition::Always,
        SignalConditionalArtifactReuse::RuntimeResolved,
    );
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let (decision, observation, rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "warm", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(4).with_label("accepted output")),
        )
        .unwrap()
        .into_parts();
    decision.unwrap();
    assert!(observation.unwrap().is_some());
    assert!(rejected.is_none());
    let before = snapshot(&mut partition, &mut graph, &contract);
    let bindings = partition
        .execute(&mut graph, |selected| {
            selected.invalidation_performed_work.shared_bindings()
        })
        .unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "finish-fault", 2)
                .force_on_demand(),
            &mut NoPredicate,
            &mut super::finalization::PoisonReuse(bindings.clone()),
            || Ok(output(4).with_label("rejected computed output")),
        );
    }))
    .is_err());
    let (reason, rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Observation { decision, cleanup } = reason else {
        panic!("execution completed before observation finalization failed");
    };
    let decision = decision.unwrap();
    assert_eq!(
        decision.class(),
        SignalConditionalDecisionClass::ComputedRevertedClean
    );
    assert_eq!(decision.counters().compute_contacts, 1);
    assert_eq!(decision.counters().decisions_delivered, 1);
    assert!(contract.retains_decision(&decision));
    assert!(cleanup.is_err());
    assert_eq!(
        rejected
            .retained_artifact_for_test(contract.node())
            .unwrap()
            .labels,
        ["rejected computed output"]
    );
    assert_restored_contents(&before, snapshot(&mut partition, &mut graph, &contract));
    assert_eq!(graph.observation_session_active_generation(), 0);

    bindings.clear_poison(); // Repair only the explicit fixture fault.
    let (decision, observation, retry_rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "retry", 3)
                .force_on_demand(),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(6)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(
        observation.unwrap().is_none(),
        "forced recomputation has no admitted invalidation work to receipt"
    );
    assert!(retry_rejected.is_none());
    assert_eq!(snapshot(&mut partition, &mut graph, &contract).0, 6);
    assert_eq!(
        rejected
            .retained_artifact_for_test(contract.node())
            .unwrap()
            .labels,
        ["rejected computed output"]
    );
}
