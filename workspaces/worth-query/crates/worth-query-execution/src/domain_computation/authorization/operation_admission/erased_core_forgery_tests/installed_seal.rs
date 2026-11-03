//! A mutually consistent tag/core seal must still match genuine installation.
use super::*;

fn forged(world: &AuthorizationWorld, request: &WorthQueryRequestScope) -> Admission {
    let (principal, account) = party(world, request);
    let mut admission = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        request
    );
    let donor = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    admission.forge_installed_seal_for_test(donor.installed_view());
    admission
}

#[test]
fn foreign_installed_seal_is_stale_at_projection_before_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let admission = forged(&world, &request);
    world.application.with_host_advancement(|phase| {
        let _work = crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        fact_work();
        let before = reads();
        let consults = observe_consults_for_test(|_| {});
        let denial = world.invariant.project_admitted_operation(
            &phase, &admission, |_, _| panic!("a stale seal never reaches the reader"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        ).err().expect("a genuine foreign seal is still stale");
        assert_eq!(denial.kind(), WorthQueryOperationProjectionDenialKind::Authorization(
            refused(AuthorizationRefusal::StaleInstalledOperation)));
        assert_eq!(reads(), before);
        assert_eq!(fact_work().source_admission_visits, 0);
        assert_eq!(consult_count_for_test(&consults), 0);
        assert!(charged_prefixes_for_test(&consults).is_empty());
    }).unwrap();
}

#[test]
fn foreign_installed_seal_is_stale_at_idempotency_before_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let admission = forged(&world, &request);
    world.application.with_host_advancement(|phase| {
        let _work = crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        fact_work();
        let before = reads();
        let consults = observe_consults_for_test(|_| {});
        let denial = world.application.resolve_admitted_application_idempotency(
            &phase, &admission, idempotency(8, 8)).unwrap_err();
        assert_eq!(denial.kind(), WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization(
            refused(AuthorizationRefusal::StaleInstalledOperation)));
        assert_eq!(reads(), before);
        assert_eq!(fact_work().source_admission_visits, 0);
        assert_eq!(consult_count_for_test(&consults), 0);
        assert!(charged_prefixes_for_test(&consults).is_empty());
    }).unwrap();
}

#[test]
fn foreign_installed_seal_is_stale_at_read_attempt_before_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let admission = forged(&world, &request);
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request();
        let execution = custody.borrowed_request_for_test();
        let _work = crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        fact_work();
        let before = reads();
        let consults = observe_consults_for_test(|_| {});
        let denial = world.application.begin_application_read_attempt(&execution, admission)
            .err().expect("a genuine foreign seal is still stale");
        assert_eq!(denial.request_authority().unwrap().kind(),
            refused(AuthorizationRefusal::StaleInstalledOperation));
        assert_eq!(reads(), before);
        assert_eq!(fact_work().source_admission_visits, 0);
        assert_eq!(consult_count_for_test(&consults), 0);
        assert!(charged_prefixes_for_test(&consults).is_empty());
    }).unwrap();
}

#[test]
fn foreign_installed_seal_is_stale_at_mutation_before_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (principal, _) = party(&world, &request);
    let execution = execute(
        &world,
        *principal.principal_identity(),
        forged(&world, &request),
    );
    assert!(execution.did_nothing());
    assert_projection_refused(
        execution.denial,
        AuthorizationRefusal::StaleInstalledOperation,
    );
}

#[test]
fn installed_input_type_is_checked_before_projection_reads() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let mut admission = admit!(
        world,
        &principal,
        &account,
        ProgramRequiredOperation,
        &request
    );
    admission.forge_tag_input_type_for_test::<u8>();
    world
        .application
        .with_host_advancement(|phase| {
            let denial = world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |_, _| panic!("a foreign input type never reaches the reader"),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .err()
                .expect("installation fixes the brand's input type");
            assert_eq!(
                denial.kind(),
                WorthQueryOperationProjectionDenialKind::Authorization(refused(
                    AuthorizationRefusal::InputNotAdmitted
                ))
            );
        })
        .unwrap();
}
