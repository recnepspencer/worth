//! Each typed program entry must invoke installed admission validation itself.
use crate::process::domain_computation::primary_graph::tests::application_attempt::erased_core_forgery::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryProgramMigrationPreparationDenial;

pub fn forged_program(
    world: &AuthorizationWorld,
) -> worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput,
    Account,
>{
    let request = live_scope();
    let (principal, account) = party(world, &request);
    let mut program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, world, &principal, &account, &request, "closed")
        })
        .unwrap();
    let donor = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    program.forge_installed_seal_for_test(donor.installed_view());
    program
}

#[test]
fn foreign_installed_seal_is_stale_at_commit_validation_before_work() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = forged_program(&world);
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let consults = observe_consults_for_test(|_| {});
    let result = world.application.validate_commit_program_for_test(program);
    let Err(WorthQueryApplicationCommitOutcome::Denied(denial)) = result else {
        panic!("foreign seal must refuse at commit validation");
    };
    assert_eq!(
        denial.authorization_denial_kind(),
        Some(refused(AuthorizationRefusal::StaleInstalledOperation))
    );
    assert_eq!(consult_count_for_test(&consults), 0);
    assert!(charged_prefixes_for_test(&consults).is_empty());
}

#[test]
fn foreign_installed_seal_is_stale_at_migration_before_work() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = forged_program(&world);
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let consults = observe_consults_for_test(|_| {});
    let result=worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::migration::prepare_program_migration_effects(&world.application,program);
    let Err(WorthQueryProgramMigrationPreparationDenial::Candidate(denial)) = result else {
        panic!("foreign seal must refuse at migration");
    };
    assert_eq!(
        denial.request_authority().unwrap().kind(),
        refused(AuthorizationRefusal::StaleInstalledOperation)
    );
    assert_eq!(consult_count_for_test(&consults), 0);
    assert!(charged_prefixes_for_test(&consults).is_empty());
}

#[test]
fn foreign_installed_seal_is_stale_at_commit_validation_in_advancement_before_work() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let program = forged_program(&world);
    world
        .application
        .with_host_advancement(|phase| {
            let _observation =
                worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
            let consults = observe_consults_for_test(|_| {});
            let result = world
                .application
                .validate_commit_program_in_advancement_for_test(&phase, program);
            let Err(WorthQueryApplicationCommitOutcome::Denied(denial)) = result else {
                panic!("the in-advancement producer must refuse the foreign seal");
            };
            assert_eq!(
                denial.authorization_denial_kind(),
                Some(refused(AuthorizationRefusal::StaleInstalledOperation))
            );
            assert_eq!(consult_count_for_test(&consults), 0);
            assert!(charged_prefixes_for_test(&consults).is_empty());
        })
        .unwrap();
}
