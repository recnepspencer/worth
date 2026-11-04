//! Production-path evidence that exact-field retention denial precedes commit.

pub use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
pub use worth_relational::facade::history::RelationalCommitReceipt;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::MultiFieldRetentionOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::WrongFieldRetentionOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

pub fn relational_head(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &AuthorizationWorld,
) -> RelationalCommitReceipt {
    world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .product()
        .relational_basis_for_test()
        .observation()
        .commit_receipt()
        .cloned()
        .expect("fixture has a Relational head")
}

pub fn assert_retention_denied(outcome: &WorthQueryApplicationCommitOutcome) {
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
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
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
                .selected_product(&(phase).test_host_request_for_test())
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
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );

    assert_retention_denied(&outcome);
    let after = world
        .application
        .with_host_advancement(|phase| relational_head(&phase, &world))
        .expect("the fixture read owns its installed host call");
    assert_eq!(after, before, "a refused inverse preimage commits nothing");
}

#[test]
fn two_field_same_record_retains_one_exact_prior_truth() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
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
                .selected_product(&(phase).test_host_request_for_test())
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
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
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
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("same-record inverse must commit: {outcome:?}");
    };
    let retained = receipt
        .retained_preimage_for_test()
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

#[path = "two_field_cross_record_retention_denial_commits_nothing/mod.rs"]
pub mod two_field_cross_record_retention_denial_commits_nothing;
