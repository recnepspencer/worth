use super::*;

pub(crate) fn expect_durability_failure(
    stop: crate::durability::data::RelationalRecoveryStop,
) -> crate::durability::data::DurabilityError {
    match stop {
        crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause) => {
            panic!("expected failed durable evidence, received admission image refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Durability(error) => error,
        crate::durability::data::RelationalRecoveryStop::Content(cause) => {
            panic!("unexpected content refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Allocation(cause) => {
            panic!("expected failed durable evidence, received allocation stop: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Request(cause) => {
            panic!("expected failed durable evidence, received request stop: {cause:?}")
        }
    }
}

/// Fork `branch_name` from main on a serial request of its own, as a test
/// fixture step whose request is irrelevant to the behavior under test.
pub(crate) fn create_branch_from_main(runtime: &RelationalRuntime, branch_name: &str) -> BranchId {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let branch_id = BranchId(branch_name.to_string());
    runtime
        .history_authority()
        .fork_branch_from(execution, branch_id.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch_id
}
