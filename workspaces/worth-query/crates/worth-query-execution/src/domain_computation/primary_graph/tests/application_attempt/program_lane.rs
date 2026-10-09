//! Native direct and producer entry refusals preserve the selected commit.

use super::program_fixture::admitted_program_required_program;
use super::*;

#[test]
fn declaration_derived_program_requirement_denies_the_raw_commit_entry() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            worth_query_declaration::facade::application_schema::TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::reference())
                    .unwrap();
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &account,
            AccountStatus::reference(),
            ProgramRequiredInput::new("program-owned").status,
        )
        .unwrap();
    let program = effects.finish().unwrap();
    let predecessor = world.selected_product().product().selected_commit().clone();

    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            program,
            idempotency(37, 37),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("a declaration-required program must deny the raw commit entry");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    let expected = format!(
        "operation {} requires the program commit lane",
        std::any::type_name::<ProgramRequiredOperation>()
    );
    assert_eq!(denial.detail(), Some(expected.as_str()));
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
}

#[test]
fn installed_program_denies_raw_commit_for_an_unlisted_operation() {
    let mut world = installed_authorization_world(true);
    assert!(!world
        .application
        .program_required_operations
        .contains(&std::any::TypeId::of::<TouchAccountOperation>()));
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let predecessor = world.selected_product().product().selected_commit().clone();
    let program = admitted_program(&world, &principal, &account, &request, "program-owned");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            program,
            idempotency(38, 38),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("an installed program must deny every raw commit entry");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    let expected = format!(
        "operation {} requires the program commit lane",
        std::any::type_name::<TouchAccountOperation>()
    );
    assert_eq!(denial.detail(), Some(expected.as_str()));
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
}

#[test]
fn program_output_producer_names_an_operation_not_installed_as_conditional() {
    let mut world = installed_authorization_world(true);
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let predecessor = world.selected_product().product().selected_commit().clone();
    let program =
        admitted_program_required_program(&world, &principal, &account, &request, "program-owned");
    let outcome = world
        .application
        .with_application_advancement(&request, |phase| {
            world
                .application
                .compare_and_commit_application_for_program_output_producer(
                    &phase,
                    program,
                    idempotency(39, 39),
                    None,
                )
        })
        .unwrap();
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("an ordinary mutation cannot enter as a conditional producer: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ApplicationProgramRequired
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    let expected = format!(
        "operation {} is not installed as a conditional operation",
        std::any::type_name::<ProgramRequiredOperation>()
    );
    assert_eq!(denial.detail(), Some(expected.as_str()));
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &predecessor
    );
}
