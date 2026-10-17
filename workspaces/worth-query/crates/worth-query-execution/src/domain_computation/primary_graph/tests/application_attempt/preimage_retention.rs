//! Production-path evidence that exact-field retention denial precedes commit.

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_relational::facade::history::RelationalCommitReceipt;

use super::{authenticated_principal, idempotency, live_scope, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountLabel, AccountStatus, AuthorizationWorld,
    MultiFieldRetentionOperation, WrongFieldRetentionOperation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

fn relational_head(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> RelationalCommitReceipt {
    world
        .selected_product(&(phase_custody).test_host_request())
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
    let before = world
        .application
        .with_host_advancement(|phase| relational_head(&phase, &world))
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(WrongFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
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
                        reader
                            .require_decision_field(projected, AccountLabel::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
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
            AccountLabel::reference(),
            "must-not-land".to_owned(),
        )
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(71, 72),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );

    assert_retention_denied(&outcome);
    let after = world
        .application
        .with_host_advancement(|phase| relational_head(&phase, &world))
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        after, before,
        "retention denial must precede Relational commit"
    );
}

#[path = "preimage_retention/two_field_cross_record_retention_denial_commits_nothing.rs"]
mod two_field_cross_record_retention_denial_commits_nothing;

#[test]
fn two_field_same_record_retains_one_exact_prior_truth() {
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
        .installed_operation(MultiFieldRetentionOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
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
                        reader
                            .require_decision_field(projected, AccountLabel::reference())
                            .unwrap();
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    let mut effects = world
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
        .write_field(&account, AccountStatus::reference(), "frozen".to_owned())
        .unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), "renamed".to_owned())
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(75, 76),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
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
