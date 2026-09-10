use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{
    CapabilityElevationStatus, CloseElevationInput, ElevatedCapabilityTouchOperation,
    RevokeCapabilityElevationOperation,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome, WorthQueryElevationCloseOutcome,
    WorthQueryElevationClosureKind, WorthQueryOperationAuthorizationDenialKind,
};

#[test]
fn approved_elevation_closes_to_a_linear_review_obligation_and_cuts_active_use() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 24));
    let requester = super::approval_transition::authenticated(&world, "alice", &request);
    let active =
        super::admit(&world, &approved, &requester, &request, Some("elevation-2")).unwrap();
    let closer = super::approval_transition::authenticated(&world, "bob", &request);

    let mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);

    assert_eq!(
        mandatory.closure_kind(),
        WorthQueryElevationClosureKind::Revoked
    );
    assert_eq!(mandatory.closer(), closer.principal_entity_id());
    assert_eq!(mandatory.publication_source().changed_record_count(), 2);
    assert_eq!(mandatory.publication_source().emitted_effect_count(), 0);
    assert_eq!(
        super::terminal_state::elevation_status(&world),
        CapabilityElevationStatus::Revoked
    );
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();
    let denial = world
        .application
        .authorize_capability_operation(active, &operation, Default::default())
        .err()
        .expect("closing must stale already-admitted active use before operation authority");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

#[test]
fn query_time_selects_expired_instead_of_accepting_caller_authored_terminal_state() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(106), 16));

    let mandatory = super::terminal_lifecycle_support::close_exact(&world, &request, approved);

    assert_eq!(
        mandatory.closure_kind(),
        WorthQueryElevationClosureKind::Expired
    );
    assert_eq!(
        super::terminal_state::elevation_status(&world),
        CapabilityElevationStatus::Expired
    );
    assert_eq!(
        mandatory.closed_at(),
        &worth_foundational::facade::AspectValue::UInt64(106)
    );
}

#[test]
fn close_command_cannot_select_a_different_elevation_subject() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 8));
    let closer = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::terminal_lifecycle_support::close_access_with_input(
        &world,
        &closer,
        &request,
        CloseElevationInput {
            account: "account-1".to_owned(),
            elevation: "elevation-1".to_owned(),
        },
    )
    .expect("the closer independently holds command authority for elevation-1");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(RevokeCapabilityElevationOperation::reference())
        .unwrap();

    let denial = world
        .application
        .authorize_elevation_close(approved, access, &operation, Default::default())
        .err()
        .expect("command authority for another subject must not close this elevation");
    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationCloseRejected
    );
}

#[test]
fn exact_relation_set_drift_stales_close_and_returns_approved_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 16));
    let requester = approved.requester();
    let program = super::terminal_lifecycle_support::materialize_close(&world, &request, approved);
    super::mutation::add_self_approver(&world, "elevation-2", requester);

    let outcome = world
        .application
        .compare_and_commit_elevation_close(program, idempotency(175, 175));
    let WorthQueryElevationCloseOutcome::Denied(denial, approved) = outcome else {
        panic!("the close bound to the prior product must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(approved.requester(), requester);
}

#[test]
fn ordinary_commit_cannot_publish_close_lifecycle_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world
        .authorization_time
        .script(std::iter::repeat_n(time(100), 16));
    let ordinary = super::terminal_lifecycle_support::close_reads(&world, &request, approved)
        .begin_effect_program()
        .finish()
        .unwrap();

    let WorthQueryApplicationCommitOutcome::Denied(denial) = world
        .application
        .compare_and_commit_application(ordinary, idempotency(176, 176))
    else {
        panic!("ordinary compare-and-commit must reject close lifecycle authority");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired
    );
}
