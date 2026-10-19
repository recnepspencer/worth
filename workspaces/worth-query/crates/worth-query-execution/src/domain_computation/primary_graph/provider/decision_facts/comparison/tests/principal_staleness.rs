use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn assert_principal_stale_and_stopped(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        requests: Vec<WorthQueryDecisionFactRequest>,
        request: &WorthQueryRequestScope,
        relation: worth_relational::facade::identity::RelationId,
    ) {
        let _listener =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        let (facts, _) = self
            .attempts
            .lock()
            .unwrap()
            .shared_observed_facts(authority.session())
            .unwrap();
        let fact = facts
            .get("application-principal-currentness")
            .expect("the real admission captures its principal");
        assert!(matches!(
            fact,
            WorthQueryPrimaryGraphApplicationDecisionFact::Principal(_)
        ));
        let snapshot = self.graph.with_runtime_mut(execution, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let mut transaction = runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .unwrap();
            use worth_relational::facade::transactions::{
                DeleteRelationIntent, MutationIntent, RelationMutationIntent, WorkerIntentBatch,
            };
            transaction
                .push_batch(
                    WorkerIntentBatch::new("retire-principal-target").push(
                        MutationIntent::Relation(RelationMutationIntent::Delete(
                            DeleteRelationIntent {
                                relation_id: relation,
                            },
                        )),
                    ),
                    execution.allocation_policy(),
                )
                .unwrap();
            transaction
                .commit(execution, runtime)
                .unwrap()
                .snapshot
                .clone()
        });
        let capture = || {
            authority
                .capture_decision_read_set(
                    requests.clone(),
                    ExecutionAllocationPolicy::SystemAllocation,
                    Some(request),
                )
                .unwrap()
        };
        let compare =
            |reader: &worth_execution::ExecutionRequest<'_, '_>,
             cancellation: Option<&worth_execution::CancellationSource>| {
                authority.compare_decision_read_set_with(capture(), |evidence, admission| {
                    let fact = facts
                        .get(evidence.locator().identity())
                        .expect("captured fact is registered");
                    let current = self
                        .graph
                        .with_runtime(reader, |runtime| {
                            if evidence.locator().identity() == "application-principal-currentness"
                                && let Some(source) = cancellation.cloned()
                            {
                                return crate::test_read_stops::cancelled_at_consult_for_test(
                                    2,
                                    source,
                                    || fact.remains_equal_in(reader, runtime, &snapshot),
                                );
                            }
                            fact.remains_equal_in(reader, runtime, &snapshot)
                        })
                        .map_err(super::super::super::fact_read_set_failure)?;
                    admission.observe_current_version(if current {
                        evidence.physical_version_evidence().to_owned()
                    } else {
                        format!("application-stale:{}", evidence.locator().identity())
                    })
                })
            };
        let WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale) =
            compare(execution, None).unwrap()
        else {
            panic!("retired principal compares stale")
        };
        assert_eq!(stale.stale_fact_count(), 1);
        assert_eq!(
            stale
                .stale_fact_identities_for_test()
                .iter()
                .map(|identity| identity.as_ref())
                .collect::<Vec<_>>(),
            ["application-observed:application-principal-currentness"]
        );
        let source = worth_execution::CancellationSource::new();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            source.token(),
            None,
        );
        let reader = worth_execution::ExecutionRequest::serial(&host);
        let stopped = compare(&reader, Some(&source));
        assert_eq!(
            stopped.err().expect("cancelled comparison stops").kind(),
            WorthQueryDecisionReadSetDenialKind::RequestInterrupted(
                WorthQueryRequestInterruption::Cancelled
            )
        );
        // Assert the dependency itself: the fact wrapper's trailing check_live
        // must not rescue a cancellation folded by freshness.
        let WorthQueryPrimaryGraphApplicationDecisionFact::Principal(dependency) = fact else {
            unreachable!()
        };
        let source = worth_execution::CancellationSource::new();
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            source.token(),
            None,
        );
        let reader = worth_execution::ExecutionRequest::serial(&host);
        let direct = self.graph.with_runtime(&reader, |runtime| {
            crate::test_read_stops::cancelled_at_consult_for_test(2, source, || {
                dependency.remains_current_in(&reader, runtime, &snapshot)
            })
        });
        assert_eq!(direct, Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))));
        self.graph.release_query_snapshot(&snapshot);
    }
}
