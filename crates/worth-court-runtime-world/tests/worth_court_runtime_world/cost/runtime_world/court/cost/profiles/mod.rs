#[allow(unused_imports)]
use crate::court::growth_context::cost::profiles::*;
pub(crate) use {
    crate::court::reference_context::cost, crate::court::reference_context::cost::observe_probe,
    crate::court::reference_context::cost::population,
    crate::court::reference_context::cost::population::Axis,
    crate::court::reference_context::cost::population::Population,
    crate::court::reference_context::cost::publication_probe,
    crate::court::reference_context::cost::writers, crate::court::reference_context::model,
    worth_runtime_world::facade::*,
};
#[test]
#[ignore = "scheduled Court scale profile"]
fn court_profile() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    profile(request_execution, "Court", [1, 4, 8], 3, 4);
}
#[test]
#[ignore = "scheduled Standard scale profile"]
fn standard_profile() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    profile(request_execution, "Standard", [1, 8, 32], 3, 6);
}
#[test]
#[ignore = "scheduled Scale profile and longer model sequence"]
fn scale_profile() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    profile(request_execution, "Scale", [1, 32, 96], 3, 8);
    crate::court::reference_context::model::driver::run(request_execution, 0x9172cafe, 32);
}
