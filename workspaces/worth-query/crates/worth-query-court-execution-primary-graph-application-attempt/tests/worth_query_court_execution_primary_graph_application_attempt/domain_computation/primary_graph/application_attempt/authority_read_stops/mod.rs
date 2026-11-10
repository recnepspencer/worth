use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{
        retained_status_program, RetentionMutationBreadth,
    };
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::{
        authenticated_principal, idempotency, resolved_account,
    };
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
        installed_authorization_world, live_scope,
    };
    use worth_query_execution::facade::primary_graph::{
        WorthQueryApplicationCommitDenialStage as Stage,
        WorthQueryApplicationCommitOutcome as Outcome,

    };
    use worth_query_execution::facade::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind as Authority,
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as Kind,
};
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
type Evidence = (Stage, Option<Result<Kind, Control>>, Option<Authority>);
fn commit_at(ceiling: u64) -> (bool, Option<Evidence>, Vec<u64>) {
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
    world.application.with_application_advancement(&scope, |phase| {
            let receiver = world.application.product_runtime().clone();
            let _regions = receiver.observe_named_read_consults_for_test();
            let selected = std::rc::Rc::new(std::cell::RefCell::new(std::collections::BTreeSet::new()));
            let indices = selected.clone();
            let observer = worth_query_execution::test_read_stops::observe_consults_for_test(move |at| {
                if receiver.authority_revalidation_is_active_for_test() {
                    indices.borrow_mut().insert(at);
                }
            });
            let execution = phase.borrowed_request_for_test();
            let (outcome, _) = execution.run(worth_execution::ExecutionWorkCeiling::new(Some(ceiling)), |_| {
                world.application.compare_and_commit_application_in_advancement(&phase, program, idempotency(202, 203), worth_execution::ExecutionAllocationPolicy::SystemAllocation)
            }).unwrap();
            let charges = worth_query_execution::test_read_stops::charged_prefixes_for_test(&observer).into_iter().filter_map(|(at, required)| selected.borrow().contains(&at).then_some(required)).collect();
            match outcome {
                Outcome::Committed(_) => (true, None, charges),
                Outcome::ProductUnpublished(effects) => {
                    assert_eq!(effects.cause(), worth_runtime_world::facade::ProductUnpublishedCause::OriginalObservationStopped(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted));
                    assert_eq!(effects.owner_effect_count(), 1);
                    assert_eq!(world.application.runtime_for_test().primary_provider_for_test().published_application_commit_count_for_test(), 0);
                    (false, None, charges)
                }
                Outcome::Denied(denial) => (false, Some((denial.stage(), denial.execution_denial_cause(), denial.authorization_denial_kind())), charges),
                other => panic!("the real commit door preserves the exact stopped fate: {other:?}"),
            }
        }).unwrap()
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
    for required in unique {
        let (committed, evidence, _) = commit_at(required - 1);
        assert!(!committed, "charge {required}");
        // Every observed charge in every actual authorization invocation is
        // selected; a non-authority gap cannot truncate a later invocation.
        assert_eq!(
            evidence,
            Some((
                Stage::DecisionReadSet,
                Some(Ok(expected)),
                Some(Authority::Stopped(Ok(expected)))
            )),
            "actual authority charge {required}"
        );
        reached += 1;
    }

    assert!(
        reached > 0,
        "the real authority revalidation read must be reached by its own charge"
    );
}
