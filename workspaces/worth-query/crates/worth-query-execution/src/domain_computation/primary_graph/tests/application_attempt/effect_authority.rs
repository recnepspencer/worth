use super::{authenticated_principal, installed_authorization_world, live_scope, resolved_account};
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::super::fixture::{
    AccountActivityEffect, AccountLabel, AccountStatus, MultiTouchOperation, TouchAccountOperation,
};

#[test]
fn compile_capability_does_not_widen_the_installed_effect_program() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(MultiTouchOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
                .authorize_operation(
                    &host_custody.phase_for_application(&world.application),
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &host_custody.phase_for_application(&world.application),
                    &admission,
                    |reader, _| {
                        let projected = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        reader
                            .require_decision_field(&projected, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut reads = world
                .application
                .begin_projected_application_read_attempt(
                    &host_custody.phase_for_application(&world.application),
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .observe_field(execution, &account, AccountStatus::reference())
                .unwrap();
            let mut effects = reads
                .complete(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
                .begin_effect_program();
            let target = effects.existing_entity(&account).unwrap();

            let Err(denial) =
                effects.write_field(&target, AccountLabel::reference(), "forged".to_string())
            else {
                panic!("a compile-capable field outside the installed program must deny");
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect
            );

            let Err(denial) = effects.emit(
                AccountActivityEffect::reference(),
                "forged-emission".to_owned(),
            ) else {
                panic!("a compile-capable emission outside the installed contract must deny");
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn entity_from_another_admitted_scope_cannot_become_an_effect_target() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let borrowed_execution = host_custody.borrowed_request_for_test();
            let execution = &borrowed_execution;
            let principal = authenticated_principal(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                &request,
            );
            let account = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "open",
                &request,
            );
            let foreign = resolved_account(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &world,
                "unrelated",
                &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            let admission = world
                .selected_product(
                    &(host_custody.phase_for_application(&world.application)).test_host_request(),
                )
                .authorize_operation(
                    &host_custody.phase_for_application(&world.application),
                    &principal,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap();
            let (_, projection, _) = world
                .invariant
                .project_admitted_operation(
                    &host_custody.phase_for_application(&world.application),
                    &admission,
                    |reader, _| {
                        let projected = reader
                            .resolve_entity(AccountStatus::reference(), "open".to_string())
                            .unwrap();
                        reader
                            .require_decision_field(&projected, AccountStatus::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts();
            let mut reads = world
                .application
                .begin_projected_application_read_attempt(
                    &host_custody.phase_for_application(&world.application),
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .observe_field(execution, &account, AccountStatus::reference())
                .unwrap();
            let effects = reads
                .complete(crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
                .begin_effect_program();

            let Err(denial) = effects.existing_entity(&foreign) else {
                panic!("a foreign admitted scope must not become a realized effect target");
            };
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
            );
        })
        .expect("the fixture block owns its installed host call")
}
