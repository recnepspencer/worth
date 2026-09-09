use super::*;
use crate::data::comparator::{
    ComparatorPolicyResolver, InstalledSignalComparatorIdentity, InstalledSignalComparatorRole,
    VersionComparatorPolicy, VersionComparatorResolver,
};
use crate::data::graph::signal_graph::PerformedWorkBuffer;
use crate::data::graph::storage::evaluation_partition::{
    SignalPartitionConditionalDenial, SignalPartitionConditionalUnwindReason,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

fn poison_capture(bindings: &Mutex<PerformedWorkBuffer>) {
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _held = bindings.lock().unwrap();
        panic!("injected capture storage failure");
    }))
    .is_err());
}

pub(super) struct PoisonReuse(pub(super) Arc<Mutex<PerformedWorkBuffer>>);

impl VersionComparatorResolver for PoisonReuse {
    fn resolve(&mut self, _: &str, _: Aspect, _: u64, _: u64) -> Result<bool, SignalError> {
        panic!("fixture uses an installed comparator")
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
        // This callback returns successfully after the injected storage fault.
        // Execution can finish; observation finishing is the first escaping panic.
        poison_capture(&self.0);
        Ok(false)
    }
}

impl ComparatorPolicyResolver for PoisonReuse {
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
fn conditional_observation_panic_preserves_completed_decision_and_restores_graph() {
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
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.unwrap().is_some());
    let bindings = partition
        .execute(&mut graph, |selected| {
            selected.invalidation_performed_work.shared_bindings()
        })
        .unwrap();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "poison-finish", 2),
            &mut NoPredicate,
            &mut PoisonReuse(bindings.clone()),
            || panic!("warm read must not compute"),
        );
    }))
    .unwrap_err();
    assert!(panic
        .downcast_ref::<String>()
        .unwrap()
        .contains("performed work observation poisoned"));
    assert_eq!(
        graph
            .node_version_for_scope(contract.node(), Aspect::new(0), None)
            .unwrap(),
        0
    );
    assert_eq!(graph.observation_session_active_generation(), 0);
    assert!(matches!(
        partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "held-evidence", 3),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || panic!("unconsumed evidence must deny before compute"),
        ),
        Err(SignalPartitionConditionalDenial::UnconsumedUnwind)
    ));
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Observation { decision, cleanup } = reason else {
        panic!("observation must be first unwind")
    };
    let decision = decision.unwrap();
    assert_eq!(
        decision.class(),
        SignalConditionalDecisionClass::DependencyUnchanged
    );
    assert_eq!(decision.counters().decisions_delivered, 1);
    assert_eq!(decision.counters().reuse_checks, 1);
    assert!(contract.retains_decision(&decision));
    assert!(cleanup.is_err());
    assert!(partition.take_conditional_unwind().is_none());
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_version_for_scope(contract.node(), Aspect::new(0), None)
                    .unwrap(),
                4
            );
            assert_eq!(selected.observation_session_active_generation(), 0);
        })
        .unwrap();
}

#[test]
fn conditional_observation_panic_preserves_returned_compute_failure() {
    let (mut graph, contract) = installed();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let bindings = partition
        .execute(&mut graph, |selected| {
            selected.invalidation_performed_work.shared_bindings()
        })
        .unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "failed-compute", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                poison_capture(&bindings);
                Err(SignalError::invalid_input("compute declined"))
            },
        );
    }))
    .is_err());
    let (reason, _rejected) = partition.take_conditional_unwind().unwrap().into_parts();
    let SignalPartitionConditionalUnwindReason::Observation { decision, cleanup } = reason else {
        panic!("observation must be first unwind")
    };
    let failure = match decision {
        Err(failure) => failure,
        Ok(_) => panic!("compute failed"),
    };
    assert_eq!(failure.counters().compute_contacts, 1);
    assert_eq!(failure.counters().decisions_delivered, 0);
    assert_eq!(
        failure.into_error(),
        SignalError::invalid_input("compute declined")
    );
    assert!(cleanup.is_err());
    assert_eq!(graph.observation_session_active_generation(), 0);
}

#[test]
fn conditional_observation_admission_panic_restores_graph_before_provider_contact() {
    let (mut graph, contract) = installed();
    let ambient_bindings = graph.invalidation_performed_work.shared_bindings();
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let bindings = partition
        .execute(&mut graph, |selected| {
            selected.invalidation_performed_work.shared_bindings()
        })
        .unwrap();
    poison_capture(&bindings);
    let computes = std::cell::Cell::new(0);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "admission-fault", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes.set(computes.get() + 1);
                Ok(output(4))
            },
        );
    }))
    .is_err());
    assert_eq!(computes.get(), 0);
    assert!(Arc::ptr_eq(
        &ambient_bindings,
        &graph.invalidation_performed_work.shared_bindings()
    ));
    assert_eq!(graph.observation_session_active_generation(), 0);
    assert!(
        partition.take_conditional_unwind().is_none(),
        "no acquired attempt to finalize"
    );
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(selected.observation_session_active_generation(), 0);
        })
        .unwrap();
    assert!(bindings.is_poisoned());
    bindings.clear_poison(); // Explicitly repair the injected fixture fault only.
    let (decision, observation, _rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "storage", "admission-retry", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes.set(computes.get() + 1);
                Ok(output(4))
            },
        )
        .unwrap()
        .into_parts();
    assert_eq!(computes.get(), 1);
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.unwrap().is_some());
}
