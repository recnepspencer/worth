//! Compiler custody fences, not evidence of runtime task or lease behavior.
#[test]
fn close_task_and_failure_custody_cannot_be_copied_or_consumed_through_borrows() {
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_task_custody_is_linear.rs"),
        "E0599", &["clone", "PhysicalStoreCloseTask"],
    ).expect("actual non-Clone task refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_task_custody_is_linear.rs"),
        "E0616", &["worker", "PhysicalStoreCloseTask", "private"],
    ).expect("actual private task custody refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_task_custody_is_linear.rs"),
        "E0382", &["PhysicalStoreCloseTask::poll", "takes ownership of the receiver", "task"],
    ).expect("actual second consuming task poll refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_task_custody_is_linear.rs"),
        "E0507", &["PhysicalStoreCloseTask::poll", "shared reference", "*task"],
    ).expect("actual borrowed task poll refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_failure_custody_is_linear.rs"),
        "E0599", &["clone", "PhysicalStoreCloseFailure"],
    ).expect("actual non-Clone failure custody refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_failure_custody_is_linear.rs"),
        "E0616", &["actual", "PhysicalStoreCloseFailure", "private"],
    ).expect("actual private failure evidence refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_failure_custody_is_linear.rs"),
        "E0382", &["PhysicalStoreCloseFailure::into_parts", "takes ownership of the receiver", "failure"],
    ).expect("actual second consuming failure extraction refusal");
    worth_testkit_compiler::certification_semantic_denial(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_replication_physical_record_journeys_physical_work/sources/worth_store/physical_runtime_authority/physical_close_failure_custody_is_linear.rs"),
        "E0507", &["PhysicalStoreCloseFailure::into_parts", "shared reference", "*failure"],
    ).expect("actual borrowed failure extraction refusal");
}
