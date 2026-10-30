pub(crate) use crate::court::ordinary_context::oracle::product::ProductModel;
pub(crate) use std::collections::BTreeMap;
pub(crate) use worth_runtime_world::facade::*;

#[test]
fn court_seeded_product_model_checks_each_transition_prefix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in [0x9172, 0x51a7, 0xcafe] {
        crate::court::reference_context::model::driver::run(request_execution, seed, 6);
    }
}
