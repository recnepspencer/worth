use super::*;
use crate::data::comparator::{
    ComparatorPolicyResolver, InstalledSignalComparatorIdentity, InstalledSignalComparatorRole,
    VersionComparatorPolicy, VersionComparatorResolver,
};
use crate::data::graph::storage::evaluation_partition::{
    SignalPartitionConditionalDenial, SignalPartitionConditionalUnwindReason,
};
use std::panic::{catch_unwind, panic_any, AssertUnwindSafe};
use std::sync::Arc;

struct PanicReuse(Arc<u64>);
struct PanicCondition(Arc<u64>);
impl InstalledSignalConditionResolver for PanicCondition {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic_any(self.0.clone())
    }
}

#[test]
fn conditional_predicate_unwind_retains_precompute_contacts() {
    let (mut graph, contract) = installed_with(
        SignalConditionalCondition::RuntimePredicate,
        SignalConditionalArtifactReuse::NotReusable,
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let payload = Arc::new(71_u64);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "predicate", 1),
            &mut PanicCondition(payload.clone()),
            &mut DefaultComparatorPolicyResolver::default(),
            || panic!("predicate failed before compute"),
        );
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        &payload,
        &unwind.downcast::<Arc<u64>>().unwrap()
    ));
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Execution {
        counters,
        observation,
        cleanup,
    } = reason
    else {
        panic!("execution must be first unwind")
    };
    assert_eq!(counters.condition_checks, 1);
    assert_eq!(counters.compute_contacts, 0);
    assert!(observation.unwrap().unwrap().is_none());
    cleanup.unwrap();
}
impl VersionComparatorResolver for PanicReuse {
    fn resolve(&mut self, _: &str, _: Aspect, _: u64, _: u64) -> Result<bool, SignalError> {
        panic!("named comparator is not installed in this fixture")
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
        panic_any(self.0.clone())
    }
}
impl ComparatorPolicyResolver for PanicReuse {
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

#[test]
fn conditional_comparator_unwind_retains_evidence_after_passive_application() {
    let (mut graph, contract) = installed_with(
        SignalConditionalCondition::Always,
        SignalConditionalArtifactReuse::RuntimeResolved,
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let (decision, observation, _rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "warm", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(4)),
        )
        .unwrap()
        .into_parts();
    decision.unwrap();
    observation.unwrap();
    let ordinal = partition
        .execute(&mut graph, |selected| {
            selected.cause_sets.output_commit_ordinal_for_test()
        })
        .unwrap();
    let payload = Arc::new(61_u64);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "reuse", 2),
            &mut NoPredicate,
            &mut PanicReuse(payload.clone()),
            || panic!("warm dependency hit must not compute"),
        );
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        &payload,
        &unwind.downcast::<Arc<u64>>().unwrap()
    ));
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Execution {
        counters,
        observation,
        cleanup,
    } = reason
    else {
        panic!("execution must be first unwind")
    };
    assert_eq!(counters.compute_contacts, 0);
    assert_eq!(counters.application_contacts, 1);
    assert_eq!(counters.reuse_checks, 1);
    assert!(observation.unwrap().unwrap().is_none());
    cleanup.unwrap();
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected.cause_sets.output_commit_ordinal_for_test(),
                ordinal
            );
            assert_eq!(
                selected
                    .node_version_for_scope(contract.node(), Aspect::new(0), None)
                    .unwrap(),
                4
            );
            assert_eq!(selected.observation_session_active_generation(), 0);
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
fn conditional_unwind_keeps_original_payload_and_report_until_single_take() {
    let (mut graph, contract) = installed();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let payload = Arc::new(31_u64);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "panic", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || panic_any(payload.clone()),
        );
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        &payload,
        &unwind.downcast::<Arc<u64>>().unwrap()
    ));
    assert_eq!(graph.observation_session_active_generation(), 0);
    assert!(matches!(
        partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "denied", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || panic!("unconsumed report must deny before providers")
        ),
        Err(SignalPartitionConditionalDenial::UnconsumedUnwind)
    ));
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Execution {
        counters,
        observation,
        cleanup,
    } = reason
    else {
        panic!("execution must be first unwind")
    };
    assert_eq!(counters.compute_contacts, 1);
    assert!(observation.unwrap().unwrap().is_none());
    cleanup.unwrap();
    assert!(partition.take_conditional_unwind().is_none());
    let (decision, observation, _rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "retry", 3),
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
}

#[test]
fn conditional_unwind_preserves_first_failure_when_observation_finish_and_drop_panic() {
    let (mut graph, contract) = installed();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let bindings = partition
        .execute(&mut graph, |selected| {
            selected.invalidation_performed_work.shared_bindings()
        })
        .unwrap();
    let payload = Arc::new(41_u64);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "poison", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                // Deliberate storage failure injection poisons the real capture owner.
                let _held = bindings.lock().unwrap();
                panic_any(payload.clone());
            },
        );
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        &payload,
        &unwind.downcast::<Arc<u64>>().unwrap()
    ));
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Execution {
        counters,
        observation,
        cleanup,
    } = reason
    else {
        panic!("execution must be first unwind")
    };
    assert_eq!(counters.compute_contacts, 1);
    assert!(observation.is_err());
    assert!(cleanup.is_err());
    assert_eq!(graph.observation_session_active_generation(), 0);
}

#[test]
fn public_conditional_entry_resumes_the_original_provider_panic() {
    let (mut graph, contract) = installed();
    let payload = Arc::new(51_u64);
    let unwind = catch_unwind(AssertUnwindSafe(|| {
        let _ = graph.execute_installed_conditional(
            SignalConditionalExecutionRequest::new(&contract, "storage", "public", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || panic_any(payload.clone()),
        );
    }))
    .unwrap_err();
    assert!(Arc::ptr_eq(
        &payload,
        &unwind.downcast::<Arc<u64>>().unwrap()
    ));
}
