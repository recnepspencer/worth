//! Literal custody refusals win even after genuine retained authentication expires.
use super::*;
use std::time::Duration;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionTimeController, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind as Authorization,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
    },
};
use worth_query_testkit_certification_operation::facade::certification::native_operation_bodies::operation_body::{
    installation::install_with_projection_on_clock, program::BodyProgram,
};

type App = worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
    Schema,
    BodyProgram<8>,
>;

fn held_scope(time: &WorthQueryAdmissionTimeController) -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(time.hold() + Duration::from_secs(7_200)),
        WorthQueryCancellationSource::new().token(),
        time.clock(),
    )
}

fn assert_expired(app: &App, program: Program) {
    let caller = caller();
    let execution = worth_execution::ExecutionRequest::serial(&caller);
    assert_eq!(execution.consult(), Ok(()));
    let outcome = app
        .runtime()
        .on_branch(app.current_world())
        .transaction()
        .apply(WorthQueryAdmittedChange::new(program, idempotency()))
        .commit_for_program(
            &execution,
            app.admit_program_operation::<SetRetention>().unwrap(),
        )
        .expect("matching custody reaches authentication revalidation");
    match outcome {
        WorthQueryApplicationCommitOutcome::Denied(denial) => assert_eq!(
            denial.authorization_denial_kind(),
            Some(Authorization::Refused(Refusal::ExpiredAuthentication)),
        ),
        other => panic!("expected literal expired authentication: {other:?}"),
    }
}

#[test]
fn program_transaction_application_mismatch_precedes_expired_authentication() {
    let time = WorthQueryAdmissionTimeController::new().unwrap();
    let scope = held_scope(&time);
    let (source, projection) = install_with_projection_on_clock::<8>(time.clock());
    let (target, _) = install_with_projection_on_clock::<8>(time.clock());
    let program = candidate(source.runtime(), &projection, &scope);
    let control = candidate(source.runtime(), &projection, &scope);
    let external = authenticate_operator(source.runtime().installed_schema(), &scope);
    let until = external.valid_until();
    assert!(!external.is_expired());
    assert_eq!(time.hold(), scope.clock().now());
    time.advance(Duration::from_secs(3_601)).unwrap();
    assert!(time.clock().now() > until);
    assert!(external.is_expired());
    assert_eq!(scope.interruption(), None);
    assert_expired(&source, control);
    let caller = caller();
    let execution = worth_execution::ExecutionRequest::serial(&caller);
    let result = target
        .runtime()
        .on_branch(target.current_world())
        .transaction()
        .apply(WorthQueryAdmittedChange::new(program, idempotency()))
        .commit_for_program(
            &execution,
            target.admit_program_operation::<SetRetention>().unwrap(),
        );
    assert!(
        matches!(
            result,
            Err(WorthQueryProductTransactionCommitError::ApplicationMismatch)
        ),
        "{result:?}"
    );
}

#[test]
fn program_transaction_branch_mismatch_precedes_expired_authentication() {
    let time = WorthQueryAdmissionTimeController::new().unwrap();
    let scope = held_scope(&time);
    let (app, projection) = install_with_projection_on_clock::<8>(time.clock());
    let sibling = app
        .with_host_advancement(|phase| {
            app.runtime()
                .branches()
                .fork(app.current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    assert_ne!(app.current_world(), sibling);
    let program = candidate(app.runtime(), &projection, &scope);
    let control = candidate(app.runtime(), &projection, &scope);
    let external = authenticate_operator(app.runtime().installed_schema(), &scope);
    let until = external.valid_until();
    assert!(!external.is_expired());
    time.advance(Duration::from_secs(3_601)).unwrap();
    assert!(time.clock().now() > until);
    assert!(external.is_expired());
    assert_eq!(scope.interruption(), None);
    assert_expired(&app, control);
    let caller = caller();
    let execution = worth_execution::ExecutionRequest::serial(&caller);
    let result = app
        .runtime()
        .on_branch(sibling)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(program, idempotency()))
        .commit_for_program(
            &execution,
            app.admit_program_operation::<SetRetention>().unwrap(),
        );
    assert!(
        matches!(
            result,
            Err(WorthQueryProductTransactionCommitError::BranchMismatch)
        ),
        "{result:?}"
    );
}
