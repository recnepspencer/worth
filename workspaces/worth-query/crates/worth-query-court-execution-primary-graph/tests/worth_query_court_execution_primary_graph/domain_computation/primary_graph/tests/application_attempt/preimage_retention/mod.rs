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
pub use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryPreImageRetentionDenial,
};

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

/// Retention runs before the Native transaction, so every retention denial is
/// a pre-effect refusal carrying its typed cause (public #122).
pub fn assert_retention_denied(
    outcome: &WorthQueryApplicationCommitOutcome,
    expected: WorthQueryPreImageRetentionDenial,
) {
    let WorthQueryApplicationCommitOutcome::Denied(denial) = outcome else {
        panic!("unexpected retention-denial outcome: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderCommit
    );
    assert_eq!(denial.preimage_retention_denial(), Some(expected));
}

#[test]
fn right_record_wrong_field_retention_denial_commits_nothing() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
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
                            .require_decision_field(projected, AccountStatus::at("AccountStatus"))
                            .unwrap();
                        reader
                            .require_decision_field(projected, AccountLabel::at("AccountLabel"))
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
            AccountLabel::at("AccountLabel"),
            "must-not-land".to_owned(),
        )
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(71, 72),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );

    // The inverse demands `AccountStatus`; the operation mutates only
    // `AccountLabel`, so no observed and mutated candidate exists for it.
    assert_retention_denied(
        &outcome,
        WorthQueryPreImageRetentionDenial::MissingDemandedField,
    );
    let after = world
        .application
        .with_host_advancement(|phase| relational_head(&phase, &world))
        .expect("the fixture read owns its installed host call");
    assert_eq!(after, before, "a refused inverse preimage commits nothing");
    next_valid_commit_publishes(&world, &before);
}

/// Commits the valid same-record operation: the two-field inverse demands
/// `AccountStatus` and `AccountLabel`, and both are mutated on one record.
pub fn commit_same_record(
    world: &AuthorizationWorld,
    key: u8,
    intent: u8,
) -> WorthQueryApplicationCommitOutcome {
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
                            .require_decision_field(projected, AccountStatus::at("AccountStatus"))
                            .unwrap();
                        reader
                            .require_decision_field(projected, AccountLabel::at("AccountLabel"))
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
            AccountStatus::at("AccountStatus"),
            "frozen".to_owned(),
        )
        .unwrap();
    effects
        .write_field(
            &account,
            AccountLabel::at("AccountLabel"),
            "renamed".to_owned(),
        )
        .unwrap();
    world.application.compare_and_commit_application(
        effects.finish().unwrap(),
        idempotency(key, intent),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

/// The check the old abort expectation stood in for: after a refused commit
/// the same application commits a valid operation next, and it publishes.
pub fn next_valid_commit_publishes(world: &AuthorizationWorld, before: &RelationalCommitReceipt) {
    let outcome = commit_same_record(world, 91, 92);
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "the application commits a valid operation after the refusal: {outcome:?}"
    );
    let head = world
        .application
        .with_host_advancement(|phase| relational_head(&phase, world))
        .expect("the fixture read owns its installed host call");
    assert_ne!(&head, before, "the valid operation publishes");
}

#[test]
fn two_field_same_record_retains_one_exact_prior_truth() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let outcome = commit_same_record(&world, 75, 76);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("same-record inverse must commit: {outcome:?}");
    };
    let retained = receipt
        .retained_preimage_for_test()
        .expect("recorded inverse retains both fields");

    assert_eq!(retained.fields().len(), 2);
    assert_eq!(
        retained
            .field_for(AccountStatus::at("AccountStatus"))
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from("open")
        )
    );
    assert_eq!(
        retained
            .field_for(AccountLabel::at("AccountLabel"))
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
