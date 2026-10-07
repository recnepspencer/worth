//! Native occurrence-owner evidence; producer entry propagation is tested separately.

use super::{require_occurrence_acts_through, resolve_occurrence_program};
use crate::domain_computation::primary_graph::{
    tests::{
        application_attempt::{admitted_operation, authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, installed_program_support, live_scope,
            rostered_program_revision, seed_program_activation, AccountStatus,
            TouchAccountOperation,
        },
    },
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
};

#[test]
fn active_program_names_an_operation_missing_from_its_declared_routes() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    seed_program_activation(&world, &rostered_program_revision());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let predecessor = world.selected_product().product().selected_commit().clone();
    let admission = admitted_operation(&world, &principal, &account, &request);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let program = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap()
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program()
        .finish()
        .unwrap();
    let support = world.application.program_support.as_ref().unwrap();
    let occurrence = resolve_occurrence_program(support, &program).unwrap();
    let denial = require_occurrence_acts_through::<TouchAccountOperation>(&occurrence).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    let expected = format!(
        "operation {} is not declared by active program worth.query.test.program-required-program.v1 revision {}",
        std::any::type_name::<TouchAccountOperation>(),
        rostered_program_revision(),
    );
    assert_eq!(denial.detail(), Some(expected.as_str()));
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
}
