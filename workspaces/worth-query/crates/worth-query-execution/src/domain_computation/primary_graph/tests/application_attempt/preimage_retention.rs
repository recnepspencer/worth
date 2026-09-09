//! Production-path evidence that exact-field retention denial precedes commit.

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::history::RelationalCommitReceipt;

use super::{authenticated_principal, idempotency, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountLabel, AccountStatus, AuthorizationWorld,
    MultiFieldRetentionOperation, WrongFieldRetentionOperation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

fn relational_head(world: &AuthorizationWorld) -> RelationalCommitReceipt {
    world
        .selected_product()
        .product()
        .relational_basis()
        .observation()
        .commit_receipt()
        .cloned()
        .expect("fixture has a Relational head")
}

fn assert_retention_denied(outcome: &WorthQueryApplicationCommitOutcome) {
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Aborted),
        "unexpected retention-denial outcome: {outcome:?}"
    );
}

#[test]
fn right_record_wrong_field_retention_denial_commits_nothing() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let before = relational_head(&world);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(WrongFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
            reader
                .require_decision_field(projected, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(
            &account,
            AccountLabel::reference(),
            "must-not-land".to_owned(),
        )
        .unwrap();
    let outcome = world
        .application
        .compare_and_commit_application(effects.finish().unwrap(), idempotency(71, 72));

    assert_retention_denied(&outcome);
    let after = relational_head(&world);
    assert_eq!(
        after, before,
        "retention denial must precede Relational commit"
    );
}

#[test]
fn two_field_cross_record_retention_denial_commits_nothing() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let unrelated = resolved_account(&world, "unrelated", &request);
    let before = relational_head(&world);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, _| {
            let open = reader
                .resolve_entity(AccountStatus::reference(), "open".to_owned())
                .unwrap();
            let unrelated = reader
                .resolve_entity(AccountStatus::reference(), "unrelated".to_owned())
                .unwrap();
            reader
                .require_decision_field(&open, AccountStatus::reference())
                .unwrap();
            reader
                .require_decision_field(&unrelated, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let open = effects.existing_entity(&account).unwrap();
    let unrelated = effects.existing_entity(&unrelated).unwrap();
    effects
        .write_field(&open, AccountStatus::reference(), "frozen".to_owned())
        .unwrap();
    effects
        .write_field(
            &unrelated,
            AccountLabel::reference(),
            "must-not-land".to_owned(),
        )
        .unwrap();
    let outcome = world
        .application
        .compare_and_commit_application(effects.finish().unwrap(), idempotency(73, 74));

    assert_retention_denied(&outcome);
    assert_eq!(relational_head(&world), before);
}

#[test]
fn two_field_same_record_retains_one_exact_prior_truth() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MultiFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountStatus::reference())
                .unwrap();
            reader
                .require_decision_field(projected, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountStatus::reference(), "frozen".to_owned())
        .unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), "renamed".to_owned())
        .unwrap();
    let outcome = world
        .application
        .compare_and_commit_application(effects.finish().unwrap(), idempotency(75, 76));
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("same-record inverse must commit: {outcome:?}");
    };
    let retained = receipt
        .retained_preimage()
        .expect("recorded inverse retains both fields");

    assert_eq!(retained.fields().len(), 2);
    assert_eq!(
        retained
            .field_for(AccountStatus::reference())
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from("open")
        )
    );
    assert_eq!(
        retained
            .field_for(AccountLabel::reference())
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from("primary")
        )
    );
    let target = retained
        .target_record()
        .expect("both retained fields name one record");
    assert!(retained
        .fields()
        .iter()
        .all(|field| field.target_record() == target));
}
