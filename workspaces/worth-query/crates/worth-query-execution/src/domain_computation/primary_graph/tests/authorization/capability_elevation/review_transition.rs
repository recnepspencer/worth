use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{CapabilityReviewStatus, CompleteElevationReviewInput};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryMandatoryReviewOutcome,
    WorthQueryOperationAuthorizationDenialKind,
};

#[test]
fn lawful_request_approve_close_review_sequence_commits_exact_distinct_actors() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 32));
    let mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);
    let reviewed = super::terminal_lifecycle_support::review_exact(&world, &request, mandatory);

    assert_eq!(reviewed.publication_source().changed_record_count(), 3);
    assert_eq!(reviewed.publication_source().emitted_effect_count(), 0);
    assert_ne!(reviewed.reviewer(), reviewed.requester());
    assert_ne!(reviewed.reviewer(), reviewed.approver());
    assert_eq!(
        reviewed.reviewed_at(),
        &worth_foundational::facade::AspectValue::UInt64(100)
    );
    assert_eq!(
        super::terminal_state::review_status(&world),
        CapabilityReviewStatus::Completed
    );
    assert!(super::terminal_state::has_exact_reviewer(
        &world,
        reviewed.reviewer()
    ));
}

#[test]
fn requester_and_approver_are_denied_by_installed_review_actor_composition() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let _mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);

    for subject in ["alice", "bob"] {
        let actor = super::approval_transition::authenticated(&world, subject, &request);
        let denial = super::terminal_lifecycle_support::review_access(&world, &actor, &request)
            .err()
            .expect("requester and approver must fail before review lifecycle authority");
        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
        );
    }
}

#[test]
fn concurrent_review_completion_stales_and_returned_obligation_cannot_be_reused() {
    let (world, request, mandatory) = denied_concurrent_review();
    let reads = super::terminal_lifecycle_support::review_reads(&world, &request, mandatory);
    let Err(denial) = reads.materialize_mandatory_review_program() else {
        panic!("completed review state must not mint another review program");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationAttemptDenialKind::MandatoryReviewProgramMismatch
    );
}

fn denied_concurrent_review() -> (
    super::approval_transition::World,
    worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    crate::domain_computation::primary_graph::WorthQueryMandatoryReview,
) {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 40));
    let mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);
    let reviewer = super::approval_transition::authenticated(&world, "carol", &request);
    let program =
        super::terminal_lifecycle_support::materialize_review(&world, &request, mandatory);
    super::mutation::complete_review_out_of_band(&world, reviewer.principal_entity_id());

    let outcome = world
        .application
        .compare_and_commit_mandatory_review(program, idempotency(177, 177));
    let WorthQueryMandatoryReviewOutcome::Denied(denial, mandatory) = outcome else {
        panic!("the review bound to the prior product must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    (world, request, mandatory)
}

#[test]
fn mandatory_review_receipt_cannot_cross_runtime_authority() {
    let (source, source_request, approved) = super::approval_transition::exact_approved_world();
    source
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let mandatory =
        super::terminal_lifecycle_support::close_exact(&source, &source_request, approved);
    let (target, target_request, _target_approved) =
        super::approval_transition::exact_approved_world();
    target
        .authorization_time
        .script(std::iter::repeat_n(time(100), 16));
    let reviewer = super::approval_transition::authenticated(&target, "carol", &target_request);
    let access =
        super::terminal_lifecycle_support::review_access(&target, &reviewer, &target_request)
            .unwrap();
    let operation = target
        .application
        .installed_schema()
        .installed_operation(
            super::super::super::fixture::CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();

    let denial = target
        .application
        .authorize_mandatory_review(mandatory, access, &operation, Default::default())
        .err()
        .expect("foreign-runtime lifecycle receipts must fail before review authority");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::MandatoryReviewRejected
    );
}

#[test]
fn mandatory_review_receipt_rejects_a_different_selected_review() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);
    let reviewer = super::approval_transition::authenticated(&world, "carol", &request);
    let access = super::terminal_lifecycle_support::review_access_with_input(
        &world,
        &reviewer,
        &request,
        CompleteElevationReviewInput {
            review: "review-1".to_owned(),
            account: "account-1".to_owned(),
            elevation: "elevation-2".to_owned(),
        },
    )
    .unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(
            super::super::super::fixture::CompleteCapabilityReviewOperation::reference(),
        )
        .unwrap();

    let denial = world
        .application
        .authorize_mandatory_review(mandatory, access, &operation, Default::default())
        .err()
        .expect("a different selected review must not consume the obligation");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::MandatoryReviewRejected
    );
}
