use crate::facade::history::BranchId;
use crate::identity::data::VersionId;
use crate::tests::support::{create_entity, install_empty_test_branch, runtime_with_test_schema};

#[test]
fn empty_branch_validation_uses_local_zero_after_unrelated_main_progress() {
    let runtime = runtime_with_test_schema();
    let empty_branch = BranchId("empty-validation".to_owned());
    install_empty_test_branch(&runtime, empty_branch.clone());

    create_entity(&runtime, "main-progress");
    let identity = runtime
        .branch_identity(&empty_branch)
        .expect("empty branch remains owner-registered");
    let transaction = {
        let transaction_validation_input = runtime
            .transaction_validation_input_for(&identity)
            .expect("owner issues the empty branch binding");
        runtime
            .begin_branch_transaction(
                transaction_validation_input.basis(),
                transaction_validation_input.intent().clone(),
            )
            .expect("owner-admitted transaction context")
    };
    let validated = transaction
        .validate(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("empty branch validation should use its local basis");

    assert_eq!(validated.validated_against_version, VersionId(0));
}
