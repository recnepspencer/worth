use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage as Stage;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome as Outcome;
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind as Authority;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as Kind;
pub type Evidence = (Stage, Option<Result<Kind, Control>>, Option<Authority>);
pub fn commit_at(ceiling: u64) -> (bool, Option<Evidence>, Vec<u64>) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let scope = live_scope();
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let principal = authenticated_principal(&host, &world, &scope);
            let account = resolved_account(&host, &world, "open", &scope);
            retained_status_program(
                &host,
                &world,
                &principal,
                &account,
                &scope,
                "authority-read-ceiling",
                RetentionMutationBreadth::Narrow,
            )
        })
        .unwrap();
    world
        .application
        .with_application_advancement(&scope, |phase| {
            let observer =
                worth_query_execution::test_read_stops::observe_consults_for_test(|_| {});
            let execution = phase.borrowed_request_for_test();
            let (outcome, _) = execution
                .run(
                    worth_execution::ExecutionWorkCeiling::new(Some(ceiling)),
                    |_| {
                        world
                            .application
                            .compare_and_commit_application_in_advancement(
                                &phase,
                                program,
                                idempotency(202, 203),
                                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                            )
                    },
                )
                .unwrap();
            let charges =
                worth_query_execution::test_read_stops::charged_prefixes_for_test(&observer)
                    .into_iter()
                    .map(|(_, required)| required)
                    .collect();
            match outcome {
                Outcome::Committed(_) => (true, None, charges),
                Outcome::Denied(denial) => (
                    false,
                    Some((
                        denial.stage(),
                        denial.execution_denial_cause(),
                        denial.authorization_denial_kind(),
                    )),
                    charges,
                ),
                other => panic!(
                    "the real commit door has a committed or pre-effect denied answer: {other:?}"
                ),
            }
        })
        .unwrap()
}
#[test]
fn authority_evidence_exposes_request_resources_by_value_through_the_commit_door() {
    let (committed, evidence, charges) = commit_at(64_000_000);
    assert!(committed);
    assert_eq!(evidence, None);
    let expected = Kind::ExecutionResource {
        denial: Resource::WorkExhausted,
        partition_identity: None,
        policy_ancestor: None,
    };
    let mut reached = 0;
    let unique: std::collections::BTreeSet<_> = charges
        .into_iter()
        .filter(|required| *required > 0)
        .collect();
    eprintln!("authority charge probes={}", unique.len());
    for (probe, required) in unique.into_iter().enumerate() {
        if probe % 100 == 0 {
            eprintln!("authority probe={probe}, required={required}, reached={reached}");
        }
        let (committed, evidence, _) = commit_at(required - 1);
        assert!(!committed, "charge {required}");
        if let Some((Stage::DecisionReadSet, _, Some(Authority::Stopped(_)))) = evidence {
            assert_eq!(
                evidence,
                Some((
                    Stage::DecisionReadSet,
                    Some(Ok(expected)),
                    Some(Authority::Stopped(Ok(expected)))
                )),
                "charge {required}"
            );
            eprintln!("authority witness probe={probe}, required={required}");
            reached += 1;
        }
    }
    assert!(
        reached > 0,
        "the real authority revalidation read must be reached by its own charge"
    );
}
