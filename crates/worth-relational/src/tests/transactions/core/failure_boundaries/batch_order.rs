use super::*;

#[test]
fn merged_plan_is_stable_across_batch_order() {
    let runtime_a = runtime_with_test_schema();
    let mut txn_a = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime_a);
        runtime_a
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn_a
        .push_batch(
            batch_create("b"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    txn_a
        .push_batch(
            batch_create("a"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let plan_a = txn_a
        .merged_plan(
            &runtime_a,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .clone();

    let runtime_b = runtime_with_test_schema();
    let mut txn_b = {
        let transaction_validation_input =
            crate::tests::support::test_owner_transaction_validation_input_for_main(&runtime_b);
        runtime_b
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    txn_b
        .push_batch(
            batch_create("a"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    txn_b
        .push_batch(
            batch_create("b"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let plan_b = txn_b
        .merged_plan(
            &runtime_b,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .clone();

    assert_eq!(plan_a, plan_b);
}
