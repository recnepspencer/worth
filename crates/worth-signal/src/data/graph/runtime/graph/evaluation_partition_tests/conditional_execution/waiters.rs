use super::*;
use crate::data::conditional_execution::SignalConditionalAttemptOutcome;
use crate::data::dependency::DependencyEdge;
use crate::data::graph::storage::evaluation_partition::ConditionalEvaluationDraft;
use crate::data::retained_storage::RetainedStoragePreparation;
use crate::facade::{mark_dirty, SignalRuntimePolicy};
use crate::tests::support::evaluate_on_demand;

fn waiting_consumer() -> (
    SignalGraph,
    InstalledSignalConditionalContract,
    crate::data::handle::NodeId,
) {
    let (mut graph, contract) = installed();
    let consumer = graph.node().build();
    for node in [contract.node(), consumer] {
        evaluate_on_demand(&mut graph, node, &mut |_, _| Ok(output(1))).unwrap();
    }
    mark_dirty(&mut graph, contract.node(), Aspect::new(0)).unwrap();
    // A real topology change while the producer is unsettled installs the
    // consumer's structural revalidation and its reverse waiter registration.
    graph
        .set_dependencies(
            consumer,
            [DependencyEdge::new(contract.node(), Aspect::new(0))],
        )
        .unwrap();
    assert!(graph.node_pending_revalidation(consumer).unwrap().is_some());
    assert_eq!(
        graph
            .node_pending_revalidation(consumer)
            .unwrap()
            .unwrap()
            .unresolved_producers(),
        [contract.node()]
    );
    (graph, contract, consumer)
}

fn visits_before_application(execution: &str) -> usize {
    let (mut graph, contract, _) = waiting_consumer();
    let mut work = RetainedStoragePreparation::new(usize::MAX);
    // Observe the production preparation cost at the provider boundary. The
    // intentionally failing provider does not claim any output application.
    graph.set_runtime_policy(
        SignalRuntimePolicy::forensic().with_maximum_waiter_resolution_visits(100_000),
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let draft = ConditionalEvaluationDraft::begin(&mut partition, &mut work).unwrap();
    let result = draft
        .partition
        .execute(&mut graph, |selected| {
            selected.execute_installed_conditional_attempt(
                SignalConditionalExecutionRequest::new(&contract, "source", execution, 1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || Err(SignalError::invalid_input("provider boundary probe")),
                &mut work,
            )
        })
        .unwrap();
    let rejected = draft.reject();
    let SignalConditionalAttemptOutcome::Completed(Err(failure)) = result else {
        panic!("probe must stop at its provider");
    };
    assert_eq!(failure.counters().compute_contacts, 1);
    drop(rejected);
    work.visits()
}

fn completed_retry_visits() -> usize {
    let (mut graph, contract, _) = waiting_consumer();
    graph.set_runtime_policy(
        SignalRuntimePolicy::forensic().with_maximum_waiter_resolution_visits(100_000),
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut work = RetainedStoragePreparation::new(usize::MAX);
    let draft = ConditionalEvaluationDraft::begin(&mut partition, &mut work).unwrap();
    let outcome = draft
        .partition
        .execute(&mut graph, |selected| {
            selected.execute_installed_conditional_attempt(
                SignalConditionalExecutionRequest::new(&contract, "source", "retry", 2),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || Ok(output(9)),
                &mut work,
            )
        })
        .unwrap();
    draft.install();
    assert!(matches!(
        outcome,
        SignalConditionalAttemptOutcome::Completed(Ok(_))
    ));
    work.visits()
}

#[test]
fn conditional_output_preparation_cannot_reset_the_attempt_and_a_fresh_attempt_retries() {
    // Size the denied coordinate from a completed native retry, so new
    // accounted work does not turn the successful twin into a policy denial.
    let retry_visits = completed_retry_visits();
    let large_execution = "execution-λ".repeat(retry_visits);
    let maximum = visits_before_application(&large_execution);
    assert!(maximum > retry_visits);
    let (mut graph, contract, consumer) = waiting_consumer();
    let previous_pending = graph.node_pending_revalidation(consumer).unwrap().cloned();
    let previous_state = graph.get_state(consumer).unwrap();
    let mut budget = graph
        .installed_runtime_policy()
        .conditional_evaluation_budget();
    budget.maximum_attempt_visits = maximum;
    graph.set_runtime_policy(
        SignalRuntimePolicy::forensic()
            .with_conditional_evaluation_budget(budget)
            .with_maximum_waiter_resolution_visits(100_000),
    );
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let mut computes = 0;
    let (decision, observation, rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "source", &large_execution, 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(9))
            },
        )
        .unwrap()
        .into_parts();
    let failure = decision
        .err()
        .expect("the attempt has no work left for output preparation");
    assert_eq!(failure.counters().compute_contacts, 1);
    assert_eq!(failure.counters().decisions_delivered, 0);
    assert_eq!(
        failure.into_error(),
        SignalError::ConditionalEvaluationWorkExhausted {
            maximum_visits: maximum
        }
    );
    assert_eq!(computes, 1);
    // The provider consumed the full allowance. Its actual compute contact
    // remains in the decision failure above; receipt construction cannot reset
    // that allowance to materialize performed targets after the denial.
    assert!(
        matches!(observation, Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }) if maximum_visits == maximum)
    );
    let rejected = rejected.expect("failed preparation retains rejected state");
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_aspect_version(contract.node())
                    .unwrap()
                    .get(Aspect::new(0)),
                1
            );
            assert_eq!(selected.get_state(consumer).unwrap(), previous_state);
            assert_eq!(
                selected.node_pending_revalidation(consumer).unwrap(),
                previous_pending.as_ref()
            );
            assert_eq!(
                selected
                    .pending_revalidation_waiters(contract.node())
                    .unwrap(),
                [consumer]
            );
        })
        .unwrap();
    // A new request uses shorter evidence coordinates, leaving its independently
    // installed allowance available for preparation and final decision storage.
    let (decision, observation, retry_rejected) = partition
        .execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "source", "retry", 2),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || Ok(output(9)),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        decision.unwrap().class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert!(observation.unwrap().is_some());
    assert!(retry_rejected.is_none());
    partition
        .execute(&mut graph, |selected| {
            assert_eq!(
                selected
                    .node_aspect_version(contract.node())
                    .unwrap()
                    .get(Aspect::new(0)),
                9
            );
            assert_eq!(
                selected.get_state(consumer).unwrap(),
                crate::data::node::NodeState::MaybeStale
            );
            // The producer is resolved, but the real topology change still
            // requires this consumer's own structural recomputation.
            let pending = selected
                .node_pending_revalidation(consumer)
                .unwrap()
                .unwrap();
            assert!(pending.is_resolved());
            assert!(pending.requires_structural_recompute());
            assert!(selected
                .pending_revalidation_waiters(contract.node())
                .unwrap()
                .is_empty());
        })
        .unwrap();
    drop(rejected);
}

#[test]
fn conditional_waiter_local_limit_still_denies_under_a_generous_attempt_allowance() {
    for maximum in [1, 100_000] {
        let (mut graph, contract, _) = waiting_consumer();
        graph.set_runtime_policy(
            SignalRuntimePolicy::forensic().with_maximum_waiter_resolution_visits(maximum),
        );
        let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
        let (decision, observation, rejected) = partition
            .execute_conditional(
                &mut graph,
                SignalConditionalExecutionRequest::new(&contract, "source", "local-waiters", 1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || Ok(output(9)),
            )
            .unwrap()
            .into_parts();
        assert!(observation.unwrap().is_some());
        if maximum == 1 {
            let failure = decision.err().expect("local waiter policy must still deny");
            assert_eq!(failure.counters().compute_contacts, 1);
            assert_eq!(
                failure.into_error(),
                SignalError::WaiterResolutionWorkExhausted { maximum_visits: 1 }
            );
            assert!(rejected.is_some());
        } else {
            assert_eq!(
                decision.unwrap().class(),
                SignalConditionalDecisionClass::ComputedChanged
            );
            assert!(rejected.is_none());
        }
    }
}
