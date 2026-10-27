use crate::facade::transactions::MutationIntent;
use crate::tests::support::*;

#[test]
fn client_key_symbol_policy_retains_preview_meaning_until_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        batch_create("intern-me"),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let plan = txn
        .prepare_merged_plan(
            &worth_execution::ExecutionRequest::serial(
                &crate::test_execution_policy::serial_request(),
            ),
            &runtime,
        )
        .unwrap()
        .clone();

    match &plan.merged_intents[0] {
        MutationIntent::Create(CreateIntent::Entity(spec)) => {
            assert!(spec.client_key.as_symbol().is_some());
        }
        other => panic!("expected create entity intent, got {other:?}"),
    }
    assert!(runtime.config().identity.symbol_table.entries.is_empty());
    let committed = txn.commit(execution, &runtime).unwrap();
    assert!(runtime
        .config()
        .identity
        .symbol_table
        .entries
        .iter()
        .any(|(_, text)| text == "intern-me"));
    release_test_commit_snapshot(&runtime, &committed);
}

#[test]
fn client_key_symbol_policy_skips_symbol_table_snapshot_refresh_when_no_raw_keys_are_present() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();
    let entity = create_entity(execution, &runtime, "stable-key");
    let before = runtime.config().identity.symbol_table.clone();

    let _ = update_entity(execution, &runtime, entity, "updated-aspect");

    assert_eq!(runtime.config().identity.symbol_table, before);
}

#[test]
fn client_key_symbol_policy_incrementally_merges_new_snapshot_entries_in_sorted_order() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .client_key_symbol_policy(ClientKeySymbolPolicy::RequireInterned)
        .build();

    let _ = create_entity(execution, &runtime, "beta");
    let _ = create_entity(execution, &runtime, "alpha");

    assert_eq!(
        runtime
            .config()
            .identity
            .symbol_table
            .entries
            .iter()
            .map(|(_, value)| value.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "beta", "lifecycle", "name"]
    );
}
