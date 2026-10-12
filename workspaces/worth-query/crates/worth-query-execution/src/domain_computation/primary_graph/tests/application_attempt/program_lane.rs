//! Native direct and producer entry refusals preserve the selected commit.

use super::program_fixture::admitted_program_required_program;
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn declaration_derived_program_requirement_denies_the_raw_commit_entry() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
        .selected_product(&(phase).test_host_request(), )
        .authorize_operation(&phase,
            &principal,
            &account,
            &operation,
            worth_query_declaration::facade::application_schema::TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, projected| {
                        reader
                            .require_decision_field(projected, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let reads = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
        .application
        .with_host_advancement(|phase| {
            reads
                .complete_projected_dependencies(
                    &phase,
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("the fixture read owns its installed host call");
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &account,
            AccountStatus::reference(),
            ProgramRequiredInput::new("program-owned").status,
        )
        .unwrap();
    let program = effects.finish().unwrap();
    let predecessor = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");

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
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &predecessor
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn installed_program_denies_raw_commit_for_an_unlisted_operation() {
    let mut world = installed_authorization_world(true);
    assert!(!world
        .application
        .program_required_operations
        .contains(&TypeId::of::<TouchAccountOperation>()));
    world.application.program_support = Some(installed_program_support(
        &world.application.installed_schema,
    ));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let predecessor = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .product()
                .selected_commit()
                .clone()
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "program-owned",
            )
        })
        .expect("the fixture read owns its installed host call");
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
    world
        .application
        .with_host_advancement(|phase| {
            assert_eq!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &predecessor
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn program_output_producer_names_an_operation_not_installed_as_conditional() {
    {
        let mut world = installed_authorization_world(true);
        world.application.program_support = Some(installed_program_support(
            &world.application.installed_schema,
        ));
        let request = live_scope();
        let principal = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                authenticated_principal(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let account = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                resolved_account(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                    &world,
                    "open",
                    &request,
                )
            })
            .expect("the fixture read owns its installed host call");
        let predecessor = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                world
                    .selected_product(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                    )
                    .product()
                    .selected_commit()
                    .clone()
            })
            .expect("the fixture read owns its installed host call");
        let program = world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                admitted_program_required_program(
                    &host_custody.phase_for_application(&world.application),
                    &world,
                    &principal,
                    &account,
                    &request,
                    "program-owned",
                )
            })
            .expect("the fixture read owns its installed host call");
        let outcome = world
            .application
            .with_application_advancement(&request, |phase| {
                let execution = &phase.borrowed_request_for_test();
                world
                    .application
                    .compare_and_commit_application_for_program_output_producer(
                        execution,
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
        world
            .application
            .with_host_advancement(|phase| {
                let host_custody = phase.test_host_request();

                assert_eq!(
                    world
                        .selected_product(
                            &(host_custody.phase_for_application(&world.application))
                                .test_host_request(),
                        )
                        .product()
                        .selected_commit(),
                    &predecessor
                )
            })
            .expect("the fixture read owns its installed host call");
    }
}
