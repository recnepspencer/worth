use crate::domain_computation::primary_graph::provider::decision_facts::comparison::tests::*;
/// Test doors on a library type, owned by the moved courts.
pub trait PrincipalStalenessWorthQueryPrimaryGraphProviderForTest: Sized {
    fn assert_principal_stale_and_stopped(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        request: &WorthQueryRequestScope,
        relation: worth_relational::facade::identity::RelationId,
    );
}

impl PrincipalStalenessWorthQueryPrimaryGraphProviderForTest for WorthQueryPrimaryGraphProvider {
    fn assert_principal_stale_and_stopped(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authority: &WorthQuerySessionReadAuthority<'_>,
        request: &WorthQueryRequestScope,
        relation: worth_relational::facade::identity::RelationId,
    ) {
        let _listener =
            worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
        let (facts, _) = (*self
            .attempts_for_test())
            .lock()
            .unwrap()
            .shared_observed_facts_for_test(authority.session_for_test())
            .unwrap();
        let requests: Vec<_> = facts.requests().collect();
        let fact = facts
            .get("application-principal-currentness")
            .expect("the real admission captures its principal");
        assert!(matches!(
            fact,
            WorthQueryPrimaryGraphApplicationDecisionFact::Principal(_)
        ));
        let snapshot = (*self.graph_for_test()).with_runtime_mut_for_test(execution, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let mut transaction = runtime
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .unwrap();
            use worth_relational::facade::transactions::DeleteRelationIntent;
            use worth_relational::facade::transactions::MutationIntent;
            use worth_relational::facade::transactions::RelationMutationIntent;
            use worth_relational::facade::transactions::WorkerIntentBatch;
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
                authority.compare_decision_read_set_with_for_test(capture(), |evidence, admission| {
                    let fact = facts
                        .get(evidence.locator().identity())
                        .expect("captured fact is registered");
                    let current = (*self
                        .graph_for_test())
                        .with_runtime(reader, |runtime| {
                            if evidence.locator().identity() == "application-principal-currentness"
                                && let Some(source) = cancellation.cloned()
                            {
                                return worth_query_execution::test_read_stops::cancelled_at_consult_for_test(
                                    2,
                                    source,
                                    || fact.remains_equal_in(reader, runtime, &snapshot),
                                );
                            }
                            fact.remains_equal_in(reader, runtime, &snapshot)
                        })
                        .map_err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::decision_facts::fact_read_set_failure)?;
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
        let direct = (*self.graph_for_test()).with_runtime(&reader, |runtime| {
            worth_query_execution::test_read_stops::cancelled_at_consult_for_test(2, source, || {
                dependency.remains_current_in_for_test(&reader, runtime, &snapshot)
            })
        });
        assert_eq!(direct, Err(worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))));
        (*self.graph_for_test()).release_query_snapshot(&snapshot);
    }
}
