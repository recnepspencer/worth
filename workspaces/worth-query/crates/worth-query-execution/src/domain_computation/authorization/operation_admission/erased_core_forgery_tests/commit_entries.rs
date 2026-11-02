//! Each typed program entry must invoke installed admission validation itself.
use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::admitted_program;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryProgramMigrationPreparationDenial,
};

fn forged_program(
    world: &AuthorizationWorld,
) -> crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    TouchAccountOperation,
    crate::domain_computation::primary_graph::tests::fixture::TouchAccountInput,
    Account,
> {
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
    let world = installed_authorization_world(true);
    let program = forged_program(&world);
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let consults = observe_consults_for_test(|_| {});
    let result = world.application.validate_commit_program(None, &program);
    let Err(WorthQueryApplicationCommitOutcome::Denied(denial)) = result else {
        panic!("foreign seal must refuse at commit validation: {result:?}");
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
    let world = installed_authorization_world(true);
    let program = forged_program(&world);
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let consults = observe_consults_for_test(|_| {});
    let result=crate::domain_computation::primary_graph::application_attempt::prepare_program_migration_effects(&world.application,program);
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
