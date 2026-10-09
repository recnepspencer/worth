use super::native_strategy_fixtures::*;

#[test]
fn validate_lowered_plan_preserves_strategy_provenance_and_commit_boundary_summary() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let lowered = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan")
    };
    let lowered_merged_plan = lowered.merged_plan().clone();
    let lowered_footprint = lowered.transaction().footprint().clone();

    let validated = {
        let mut authority = CommitStrategiesAuthorityFacade::new();
        authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated lowered strategy plan")
    };

    let artifacts = validated
        .strategy_commit_artifacts()
        .expect("strategy validation decorates the canonical proposal");
    assert_eq!(
        artifacts.lowering_provenance().strategy_id(),
        CommitStrategyId(41)
    );
    assert_eq!(validated.validated_against_version_id().0, 0);
    assert_eq!(validated.prepared.merged_plan, lowered_merged_plan);
    assert_eq!(validated.footprint(), &lowered_footprint);
    assert!(validated.validation_summary().commit_boundary_seen);
    assert!(validated.validation_summary().mutation_sensitive_seen);
    assert!(validated.validation_summary().snapshot_publication_seen);
    assert_eq!(validated.validation_summary().execution_count, 3);
    assert!(validated.validation_summary().plan_backed_execution_count >= 1);
    let preview_cost = artifacts
        .preview_validation_cost()
        .expect("strategy proposal records preview validation cost");
    assert_eq!(preview_cost.merged_intent_count(), 1);
    assert_eq!(preview_cost.post_mutation_preview_pass_count(), 2);
}

#[test]
fn validated_strategy_proposal_keeps_its_unique_version_across_unrelated_sibling_advance() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();

    let mut seed = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    seed.push_batch(
        WorkerIntentBatch::new("main-before-fork").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId(1),
                kind_id: KindId(1),
                client_key: ClientKey::from("main-before-fork"),
                fields: strategy_name_and_replicas_patch("main-before-fork", 1),
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    seed.commit(
        &runtime,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("pre-fork main commit");
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(BranchId("strategy-version-child".to_owned()), basis)
        .expect("child branch installs from main");

    let request = canonical_request();
    let execution = execution_draft(&request);
    let child = BranchId("strategy-version-child".to_owned());
    let validated = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, Some(child));
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("child strategy lowers against its selected root");
        authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("child strategy validates before sibling advance")
    };
    let validated_version = validated.proposal_identity().proposed_version_id();

    let mut sibling = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    sibling
        .push_batch(
            WorkerIntentBatch::new("main-after-validation").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId(1),
                    kind_id: KindId(1),
                    client_key: ClientKey::from("main-after-validation"),
                    fields: strategy_name_and_replicas_patch("main-after-validation", 1),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    sibling
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("unrelated sibling commit");

    assert_ne!(
        validated_version,
        runtime.history().preview_next_version_id(),
        "the sibling commit advances the allocator beyond the reserved proposal version"
    );

    let commit = {
        let mut authority = CommitStrategiesAuthorityFacade::new();
        authority
            .execute_validated_commit(&runtime, validated)
            .expect("unrelated sibling progress remains admissible")
    };
    assert_eq!(commit.version_id, validated_version);
}

#[test]
fn execute_validated_commit_routes_prevalidated_strategy_plan_through_authoritative_pipeline() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let validated = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan");
        authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated lowered strategy plan")
    };

    let commit = {
        let mut authority = CommitStrategiesAuthorityFacade::new();
        authority
            .execute_validated_commit(&runtime, validated)
            .expect("validated strategy commit executed")
    };

    assert_eq!(commit.commit.commit_id.0, 1);
    assert!(commit.validation().summary.execution_count >= 3);
    assert!(commit.validation().summary.commit_boundary_seen);
    let strategy_artifacts = commit
        .publication()
        .strategy_artifacts
        .as_ref()
        .expect("strategy artifacts on publication");
    assert!(strategy_artifacts.preview_validation_summary().is_some());
    assert_eq!(
        strategy_artifacts
            .preview_validation_cost()
            .expect("preview validation cost")
            .post_mutation_preview_pass_count(),
        2
    );
    assert_eq!(
        strategy_artifacts
            .merge_descriptor()
            .merge_semantics()
            .conflict_class(),
        crate::commit_strategies::data::StrategyMergeConflictClass::IntentReconciliation
    );
    assert_eq!(
        commit.publication().envelope.strategy_artifacts.as_ref(),
        Some(strategy_artifacts)
    );
}

#[test]
fn validate_lowered_plan_rejects_stale_basis_with_zero_residue() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let lowered = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan")
    };
    let mut intervening = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    intervening
        .push_batch(
            WorkerIntentBatch::new("advance-before-validation").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId(1),
                    kind_id: KindId(1),
                    client_key: ClientKey::from("advance-before-validation"),
                    fields: strategy_name_and_replicas_patch("advance-before-validation", 1),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    intervening
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("intervening commit advances the exact branch");
    let symbols_before = runtime.services.symbols.clone();
    let configured_symbols_before = runtime.config().identity.symbol_table.clone();
    let branch_cells_before = runtime.history().branch_cells_snapshot();
    let catalog_before = runtime.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime.phase4_reference_cost_counters();
    let complexity_before = runtime.performance_access().counters();

    let error = CommitStrategiesAuthorityFacade::new()
        .validate_lowered_plan(&runtime, lowered)
        .expect_err("validation rejects a lowered plan after its branch moves");

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::transactions::data::ConflictClass::StaleValidationBasis { .. }
            )
    ));
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(
        runtime.config().identity.symbol_table,
        configured_symbols_before
    );
    assert_eq!(
        runtime.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        catalog_before
    );
}

#[test]
fn execute_validated_commit_rejects_stale_validation_basis_after_intervening_commit() {
    let runtime = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let validated = {
        let (transaction_validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime, None);
        let lowered = authority
            .lower_execution_with_input(
                &runtime,
                &request,
                &execution,
                transaction_validation_input,
            )
            .expect("lowered strategy plan");
        authority
            .validate_lowered_plan(&runtime, lowered)
            .expect("validated lowered strategy plan")
    };

    let mut ordinary_txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    ordinary_txn
        .push_batch(
            WorkerIntentBatch::new("ordinary-create").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId(1),
                    kind_id: KindId(1),
                    client_key: ClientKey::from("ordinary-a"),
                    fields: strategy_name_and_replicas_patch("ordinary-a", 1),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    ordinary_txn
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("ordinary commit succeeds");
    let symbols_before = runtime.services.symbols.clone();
    let configured_symbols_before = runtime.config().identity.symbol_table.clone();
    let branch_cells_before = runtime.history().branch_cells_snapshot();
    let catalog_before = runtime.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime.phase4_reference_cost_counters();
    let complexity_before = runtime.performance_access().counters();

    let error = {
        let mut authority = CommitStrategiesAuthorityFacade::new();
        authority
            .execute_validated_commit(&runtime, validated)
            .expect_err("stale validated strategy plan should be rejected")
    };

    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert!(matches!(
                error.class,
                crate::transactions::data::ConflictClass::StaleValidationBasis { .. }
            ));
        }
        other => panic!("expected conflict rejection, got {other:?}"),
    }
    assert_eq!(runtime.services.symbols, symbols_before);
    assert_eq!(
        runtime.config().identity.symbol_table,
        configured_symbols_before
    );
    assert_eq!(
        runtime.phase4_reference_cost_counters(),
        reference_cost_before
    );
    assert_eq!(runtime.performance_access().counters(), complexity_before);
    assert_eq!(
        runtime.history().branch_cells_snapshot(),
        branch_cells_before
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        catalog_before
    );
}
