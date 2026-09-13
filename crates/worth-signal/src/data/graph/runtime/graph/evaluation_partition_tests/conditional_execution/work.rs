use super::*;
use crate::data::graph::storage::evaluation_partition::{
    ConditionalEvaluationDraft, SignalPartitionConditionalDenial,
};
use crate::facade::SignalRuntimePolicy;

#[test]
fn selected_attempt_work_is_not_replenished_before_application_or_finalization() {
    // Observe one completed execution's work, then require that exact allowance
    // through the retained kernel. The short twin fails during final evidence
    // construction, after output application. Fork/mutation byte reservation
    // is still a separate unfinished milestone obligation.
    let (draft, engine, exact) = completed_attempt_visits();
    for maximum in [draft - 1, engine - 1, exact - 1, exact] {
        let (mut graph, contract) = installed();
        let mut budget = graph
            .installed_runtime_policy()
            .conditional_evaluation_budget();
        budget.maximum_attempt_visits = maximum;
        graph.set_runtime_policy(
            SignalRuntimePolicy::forensic().with_conditional_evaluation_budget(budget),
        );
        // This fixture isolates execution allowance from initial slot admission.
        // Use explicit bounded setup work; the installed attempt budget remains
        // the short allowance exercised below, including its one-visit twin.
        let mut setup_work =
            crate::data::retained_storage::RetainedStoragePreparation::new(100_000);
        let basis = crate::data::graph::storage::execution_basis::SignalExecutionBasis::capture(
            &mut graph,
            &mut setup_work,
        )
        .unwrap();
        let mut partition = basis.try_new_evaluation_partition(&mut setup_work).unwrap();
        // Ambient policy movement cannot grant the retained definition a fresh
        // or larger allowance when its storage is activated.
        graph.set_runtime_policy(SignalRuntimePolicy::forensic());
        let mut computes = 0;
        let completion = partition.execute_conditional(
            &mut graph,
            SignalConditionalExecutionRequest::new(&contract, "selected", "budget", 1),
            &mut NoPredicate,
            &mut DefaultComparatorPolicyResolver::default(),
            || {
                computes += 1;
                Ok(output(9).with_label("attempt output"))
            },
        );
        if maximum < draft {
            assert!(matches!(
                completion,
                Err(SignalPartitionConditionalDenial::StorageAdmission(
                    SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }
                )) if maximum_visits == maximum
            ));
            assert_eq!(computes, 0);
        } else {
            let (decision, observation, rejected) = completion.unwrap().into_parts();
            if maximum == exact {
                assert_eq!(
                    decision.unwrap().class(),
                    SignalConditionalDecisionClass::ComputedChanged
                );
                assert_eq!(computes, 1);
                assert!(observation.unwrap().is_some());
                assert!(rejected.is_none());
            } else if maximum == exact - 1 {
                assert_eq!(
                    decision.unwrap().class(),
                    SignalConditionalDecisionClass::ComputedChanged
                );
                assert_eq!(computes, 1);
                assert!(
                    matches!(observation, Err(SignalError::ConditionalEvaluationWorkExhausted { maximum_visits }) if maximum_visits == maximum)
                );
                assert!(
                    rejected.is_some(),
                    "receipt denial retains the performed candidate"
                );
            } else {
                let failure = decision.err().expect("installed allowance must deny");
                assert_eq!(failure.counters().compute_contacts, 1);
                assert_eq!(
                    failure.into_error(),
                    SignalError::ConditionalEvaluationWorkExhausted {
                        maximum_visits: maximum
                    }
                );
                let rejected = rejected.expect("failed attempt retains its rejected state");
                assert_eq!(computes, 1);
                assert!(observation.unwrap().is_some());
                assert_eq!(
                    rejected
                        .retained_artifact_for_test(contract.node())
                        .unwrap()
                        .labels,
                    ["attempt output"]
                );
            }
        }
        partition
            .execute(&mut graph, |selected| {
                assert_eq!(
                    selected
                        .node_version_for_scope(contract.node(), Aspect::new(0), None)
                        .unwrap(),
                    if maximum == exact { 9 } else { 0 }
                );
            })
            .unwrap();
    }
}

fn completed_attempt_visits() -> (usize, usize, usize) {
    let (mut graph, contract) = installed();
    graph.set_runtime_policy(SignalRuntimePolicy::forensic());
    let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(1_000_000);
    // Measure the retained storage representation used by the native twin.
    // Its immutable base plus overlay require different lookup work from an
    // exclusive graph, even when their logical dependency sets are identical.
    let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
    let draft = ConditionalEvaluationDraft::begin(&mut partition, &mut work).unwrap();
    let draft_visits = work.visits();
    let (result, engine) = draft
        .partition
        .execute(&mut graph, |selected| {
            let observation = selected
                .begin_observation_session(
                    crate::logic::transaction::SignalObservationRequest::operation(),
                )
                .unwrap();
            let result = selected.execute_installed_conditional_attempt(
                SignalConditionalExecutionRequest::new(&contract, "selected", "budget", 1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || Ok(output(9).with_label("attempt output")),
                &mut work,
            );
            let engine = work.visits();
            assert!(selected
                .finish_optional_observation_session_with_work(
                    &observation,
                    &mut crate::logic::evaluation::EvaluationWork::Conditional(&mut work)
                )
                .unwrap()
                .is_some());
            (result, engine)
        })
        .unwrap();
    draft.install();
    assert!(matches!(
        result,
        crate::data::conditional_execution::SignalConditionalAttemptOutcome::Completed(Ok(_))
    ));
    (draft_visits, engine, work.visits())
}
