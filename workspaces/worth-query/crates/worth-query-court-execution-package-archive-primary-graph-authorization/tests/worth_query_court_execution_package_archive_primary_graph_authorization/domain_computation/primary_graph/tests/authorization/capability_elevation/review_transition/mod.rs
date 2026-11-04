use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::CapabilityReviewStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::elevated::review::CompleteElevationReviewInput;
use crate::domain_computation::primary_graph::tests::authorization::capability_progression::time;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryMandatoryReviewOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

#[test]
fn lawful_request_approve_close_review_sequence_commits_exact_distinct_actors() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 32));
    let mandatory = {
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_exact(&world, &request, approved)
    };
    let reviewed = {
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::review_exact(&world, &request, mandatory)
    };

    assert_eq!(reviewed.publication_source().changed_record_count(), 3);
    assert_eq!(reviewed.publication_source().emitted_effect_count(), 0);
    assert_ne!(reviewed.reviewer(), reviewed.requester());
    assert_ne!(reviewed.reviewer(), reviewed.approver());
    assert_eq!(
        reviewed.reviewed_at(),
        &worth_foundational::facade::AspectValue::UInt64(100)
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            assert_eq!(
                crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_state::review_status(
                    &host_custody,
                    &host_custody.borrowed_request_for_test(),
                    &world
                ),
                CapabilityReviewStatus::Completed
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            assert!(crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_state::has_exact_reviewer(
                &host_custody,
                execution,
                &world,
                reviewed.reviewer()
            ))
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn requester_and_approver_are_denied_by_installed_review_actor_composition() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let _mandatory = {
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_exact(&world, &request, approved)
    };

    for subject in ["alice", "bob"] {
        let actor = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(&host_custody, &world, subject, &request)
            })
            .expect("the fixture read owns its installed host call");
        let denial = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request_for_test();
                crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::review_access(
                    &host_custody,
                    &world,
                    &actor,
                    &request,
                )
                .err()
                .expect("requester and approver must fail before review lifecycle authority")
            })
            .expect("the fixture read owns its installed host call");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityAuthorizationMissing
            )
        );
    }
}

#[test]
fn concurrent_review_completion_stales_and_returned_obligation_cannot_be_reused() {
    let (world, request, mandatory) = { denied_concurrent_review() };
    let reads = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::review_reads(
                &host_custody,
                &world,
                &request,
                mandatory,
            )
        })
        .expect("the fixture read owns its installed host call");
    let Err(denial) = reads.materialize_mandatory_review_program() else {
        panic!("completed review state must not mint another review program");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::MandatoryReviewProgramMismatch
    );
}

pub fn denied_concurrent_review() -> (
    crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    worth_query_execution::facade::domain_computation::WorthQueryMandatoryReview,
){
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 40));
    let mandatory = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_exact(&world, &request, approved);
    let (reviewer, program) = world
        .application
        .with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request_for_test();
            let reviewer =
                crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(&custody, &world, "carol", &request);
            let program = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::materialize_review(
                &custody, &world, &request, mandatory,
            );
            (reviewer, program)
        })
        .expect("the review materialization owns its host call");
    world
        .application
        .with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request_for_test();
            let phase_custody = &custody;
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::review_support_mutation::complete_review_out_of_band(
                phase_custody,
                &phase_custody.borrowed_request_for_test(),
                &world,
                reviewer.principal_entity_id_for_test(),
            );
        })
        .expect("the concurrent mutation owns its host call");

    let outcome = world
        .application
        .compare_and_commit_mandatory_review(program, idempotency(177, 177));
    let WorthQueryMandatoryReviewOutcome::Stale(stale, mandatory) = outcome else {
        panic!("the changed review facts must stale before effects: {outcome:?}");
    };
    assert!(stale.stale_fact_count() > 0);
    (world, request, mandatory)
}

#[test]
fn mandatory_review_receipt_cannot_cross_runtime_authority() {
    let (source, source_request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    source
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let mandatory = {
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_exact(&source, &source_request, approved)
    };
    let (target, target_request, _target_approved) =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    target
        .authorization_time
        .script(std::iter::repeat_n(time(100), 16));
    let reviewer = target
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(
                &host_custody,
                &target,
                "carol",
                &target_request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let access = target
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::review_access(
                &host_custody,
                &target,
                &reviewer,
                &target_request,
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = target
        .application
        .installed_schema()
        .installed_operation(
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();

    let denial = target
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            target
                .application
                .authorize_mandatory_review(
                    &host_custody.phase_for_application(&target.application),
                    mandatory,
                    access,
                    &operation,
                    Default::default(),
                )
                .err()
                .expect("foreign-runtime lifecycle receipts must fail before review authority")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::MandatoryReviewRejected
        )
    );
}

#[test]
fn mandatory_review_receipt_rejects_a_different_selected_review() {
    let (world, request, approved) = crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let mandatory = {
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::close_exact(&world, &request, approved)
    };
    let reviewer = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(&host_custody, &world, "carol", &request)
        })
        .expect("the fixture read owns its installed host call");
    let access = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::review_access_with_input(
                &host_custody,
                &world,
                &reviewer,
                &request,
                CompleteElevationReviewInput {
                    review: "review-1".to_owned(),
                    account: "account-1".to_owned(),
                    elevation: "elevation-2".to_owned(),
                },
            )
            .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();

    let denial = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            world
                .application
                .authorize_mandatory_review(
                    &host_custody.phase_for_application(&world.application),
                    mandatory,
                    access,
                    &operation,
                    Default::default(),
                )
                .err()
                .expect("a different selected review must not consume the obligation")
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::MandatoryReviewRejected
        )
    );
}
