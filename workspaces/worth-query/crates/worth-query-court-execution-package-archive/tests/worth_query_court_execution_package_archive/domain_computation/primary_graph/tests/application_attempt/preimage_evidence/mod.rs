//! Exact reusable commit evidence and demanded-work breadth proofs.

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::assert_retained_status;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::demanded_mutation_work;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;

pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRole;
pub use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
pub use worth_query_declaration::facade::application_operation::WorthQueryExactlyOneOutput;
pub use worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput;
pub use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::role_use::OutputRoleUse;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::RetainedStatusEffect;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::RetainedStatusNotice;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
#[test]
fn later_commit_recovery_selects_exact_evidence_and_rejects_foreign_reference_axes() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let first_account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request_for_test(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let later_account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request_for_test(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let first = world
        .application
        .with_host_advancement(|phase| {
            retained_status_program(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &first_account,
                &request,
                "first-commit",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    let first_outcome = world.application.compare_and_commit_application(
        first,
        idempotency(85, 86),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(
            first_outcome,
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "first exact-evidence commit failed: {first_outcome:?}"
    );

    let later = world
        .application
        .with_host_advancement(|phase| {
            retained_status_program(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &later_account,
                &request,
                "later-commit",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            retained_status_program(
                &(phase).test_host_request_for_test(),
                &world,
                &principal,
                &later_account,
                &request,
                "later-commit",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryApplicationCommitOutcome::Committed(later_receipt) =
        world.application.compare_and_commit_application(
            later,
            idempotency(87, 88),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the later exact-evidence target commits");
    };
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(87, 88),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the later commit must not select the first stored entry");
    };
    assert!(recovered.is_same_authoritative_commit(&later_receipt));
    assert_retained_status(&recovered, "unrelated");

    let mut foreign_version = later_receipt.commit_reference().clone();
    foreign_version.version_id =
        worth_relational::facade::identity::VersionId::new(foreign_version.version_id.as_u64() + 1);
    let mut foreign_branch = later_receipt.commit_reference().clone();
    foreign_branch.branch_id =
        worth_relational::facade::history::BranchId("foreign-branch".to_owned());
    let mut foreign_parents = later_receipt.commit_reference().clone();
    foreign_parents.parents.push(foreign_parents.commit_id);
    for foreign in [foreign_version, foreign_branch, foreign_parents] {
        assert!(
            (*world
                .application
                .primary_provider_for_test())
                .observe_completed_application(&foreign)
                .is_none(),
            "equal CommitId with any foreign RelationalCommitReceipt axis must not observe evidence"
        );
    }
}
#[test]
fn demanded_work_tracks_real_multi_intent_and_target_breadth() {
    {
        let narrow = demanded_mutation_work(RetentionMutationBreadth::Narrow, 91);
        let wide = demanded_mutation_work(RetentionMutationBreadth::CrossRecordLabel, 92);

        assert_eq!(narrow.preimage_demanded_loci_examined(), 1);
        assert_eq!(wide.preimage_demanded_loci_examined(), 1);
        assert_eq!(narrow.preimage_mutation_targets_materialized(), 1);
        assert_eq!(wide.preimage_mutation_targets_materialized(), 2);
        assert_eq!(
            narrow.preimage_candidates_materialized(),
            narrow.preimage_demanded_loci_examined()
        );
        assert_eq!(
            wide.preimage_candidates_materialized(),
            narrow.preimage_candidates_materialized(),
            "an unrelated record/field mutation supplies no demanded preimage"
        );
        assert_eq!(
            narrow.preimage_decision_facts_examined(),
            narrow.preimage_candidates_materialized()
        );
        assert_eq!(
            wide.preimage_decision_facts_examined(),
            wide.preimage_candidates_materialized()
        );
        assert_eq!(narrow.preimage_validated_intents_examined(), 3);
        assert_eq!(wide.preimage_validated_intents_examined(), 4);
    }
}

#[path = "response_loss_and_interleaving_preserve_one_preimage_and_outbox/mod.rs"]
pub mod response_loss_and_interleaving_preserve_one_preimage_and_outbox;
