pub(super) mod driver;
mod observation;
mod publication;
use super::oracle::product::ProductModel;
use super::*;
use std::collections::BTreeMap;

#[test]
fn court_seeded_product_model_checks_each_transition_prefix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in [0x9172, 0x51a7, 0xcafe] {
        driver::run(request_execution, seed, 6);
    }
}
