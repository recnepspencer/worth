//! Presence and absence from the actual admitted snapshot remain decision facts.
use super::super::{
    admitted_operation, admitted_program_with_expected_status, assert_changed_decision,
    authenticated_principal, idempotency, resolved_account,
};
use crate::domain_computation::primary_graph::{
    tests::fixture::{
        installed_authorization_world, live_scope, Account, AccountNote, AccountStatus,
        AuthorizationWorld, IdentityExecutionSchema, PatchAccountDraftInput,
        PatchAccountDraftOperation,
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
};
use std::cell::Cell;

#[test]
fn present_predecode_field_stales_after_a_competing_actual_change() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let admission = admitted_operation(&world, &principal, &account, &request);
    let (_, snapshot, work) = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            let value = reader
                .decision_field_with_predecode_admission(
                    root,
                    AccountStatus::reference(),
                    |_, _| Ok::<_, ()>(()),
                    &|| Ok(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(value.as_deref(), Some("open"));
        })
        .unwrap()
        .into_parts();
    assert_eq!(work.field_reads(), 1);
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, snapshot)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&target, AccountStatus::reference(), "loser".to_owned())
        .unwrap();
    let loser = effects.finish().unwrap();
    let winner = admitted_program_with_expected_status(
        &world,
        &principal,
        &account,
        &request,
        ("open", "winner"),
    );
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(201, 201)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_changed_decision(
        world
            .application
            .compare_and_commit_application(loser, idempotency(202, 202)),
        "the admitted predecode source field changed",
    );
}

#[test]
fn absence_skips_admission_and_stales_after_a_competing_presence_change() {
    let world = installed_authorization_world(true);
    let called = Cell::new(false);
    let loser = optional_program(&world, "loser", &called);
    assert!(!called.get(), "absence never invokes scalar admission");
    let winner = optional_program(&world, "winner", &called);
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(winner, idempotency(203, 203)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    assert_changed_decision(
        world
            .application
            .compare_and_commit_application(loser, idempotency(204, 204)),
        "the predecode-observed absent field became present",
    );
}

fn optional_program(
    world: &AuthorizationWorld,
    note: &str,
    called: &Cell<bool>,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    PatchAccountDraftOperation,
    PatchAccountDraftInput,
    Account,
> {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "unrelated", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(PatchAccountDraftOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            Default::default(),
            &request,
        )
        .unwrap();
    let (_, snapshot, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, root| {
            let value = reader
                .decision_field_with_predecode_admission(
                    root,
                    AccountNote::reference(),
                    |_, _| {
                        called.set(true);
                        Ok::<_, ()>(())
                    },
                    &|| Ok(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(value, None);
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, snapshot)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let target = effects.existing_entity(&account).unwrap();
    effects
        .write_optional_field(&target, AccountNote::reference(), Some(note.to_owned()))
        .unwrap();
    effects.finish().unwrap()
}
