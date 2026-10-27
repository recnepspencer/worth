use super::native_strategy_fixtures::*;

macro_rules! assert_target_state_unchanged {
    ($runtime:expr, $symbols:expr, $configured:expr, $cells:expr, $catalog:expr, $reference:expr, $complexity:expr $(,)?) => {{
        assert_eq!($runtime.services.symbols, $symbols);
        assert_eq!($runtime.config().identity.symbol_table, $configured);
        assert_eq!($runtime.phase4_reference_cost_counters(), $reference);
        assert_eq!($runtime.performance_access().counters(), $complexity);
        assert_eq!($runtime.history().branch_cells_snapshot(), $cells);
        assert_eq!($runtime.history().commit_envelopes_snapshot(), $catalog);
    }};
}

#[test]
fn validate_lowered_plan_denies_foreign_runtime_before_missing_strategy_lookup() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime_a = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let lowered = {
        let (validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime_a, None);
        authority
            .lower_execution_with_input(
                request_execution,
                &runtime_a,
                &request,
                &execution,
                validation_input,
            )
            .expect("source runtime lowers its strategy plan")
    };
    let runtime_b = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .build();
    let symbols_before = runtime_b.services.symbols.clone();
    let configured_symbols_before = runtime_b.config().identity.symbol_table.clone();
    let branch_cells_before = runtime_b.history().branch_cells_snapshot();
    let catalog_before = runtime_b.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime_b.phase4_reference_cost_counters();
    let complexity_before = runtime_b.performance_access().counters();

    let error = CommitStrategiesAuthorityFacade::new()
        .validate_lowered_plan(request_execution, &runtime_b, lowered)
        .expect_err("lowered strategy authority cannot cross runtimes");

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::transactions::data::ConflictClass::ForeignRuntime { .. }
            )
    ));
    assert_target_state_unchanged!(
        &runtime_b,
        symbols_before,
        configured_symbols_before,
        branch_cells_before,
        catalog_before,
        reference_cost_before,
        complexity_before,
    );
}

#[test]
fn execute_validated_commit_preserves_foreign_runtime_taxonomy_and_target_state() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime_a = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let request = canonical_request();
    let execution = execution_draft(&request);
    let validated = {
        let (validation_input, mut authority) =
            crate::tests::support::test_owner_strategy_authority(&runtime_a, None);
        let lowered = authority
            .lower_execution_with_input(
                request_execution,
                &runtime_a,
                &request,
                &execution,
                validation_input,
            )
            .expect("source runtime lowers its strategy plan");
        authority
            .validate_lowered_plan(request_execution, &runtime_a, lowered)
            .expect("source runtime validates its strategy plan")
    };
    let runtime_b = RelationalRuntimeBuilder::new()
        .schema_registry(strategy_schema_registry())
        .commit_strategy(strategy_registration())
        .build();
    let symbols_before = runtime_b.services.symbols.clone();
    let configured_symbols_before = runtime_b.config().identity.symbol_table.clone();
    let branch_cells_before = runtime_b.history().branch_cells_snapshot();
    let catalog_before = runtime_b.history().commit_envelopes_snapshot();
    let reference_cost_before = runtime_b.phase4_reference_cost_counters();
    let complexity_before = runtime_b.performance_access().counters();

    let error = CommitStrategiesAuthorityFacade::new()
        .execute_validated_commit(request_execution, &runtime_b, validated)
        .expect_err("validated strategy authority cannot cross runtimes");

    assert!(matches!(
        error,
        TransactionCommitError::Conflict { error, .. }
            if matches!(
                error.class,
                crate::transactions::data::ConflictClass::ForeignRuntime { .. }
            )
    ));
    assert_target_state_unchanged!(
        &runtime_b,
        symbols_before,
        configured_symbols_before,
        branch_cells_before,
        catalog_before,
        reference_cost_before,
        complexity_before,
    );
}
