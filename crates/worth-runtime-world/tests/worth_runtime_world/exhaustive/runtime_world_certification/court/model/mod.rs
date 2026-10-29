#[path = "../../../../fixtures/runtime_world_certification/court/model/driver.rs"]
pub(crate) mod driver;

#[path = "../../../../fixtures/runtime_world_certification/court/model/observation.rs"]
pub(crate) mod observation;

#[path = "../../../../fixtures/runtime_world_certification/court/model/publication.rs"]
pub(crate) mod publication;

pub(crate) use super::oracle::product::ProductModel;
pub(crate) use super::*;
pub(crate) use std::collections::BTreeMap;

#[test]
fn court_seeded_product_model_checks_each_transition_prefix() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for seed in [0x9172, 0x51a7, 0xcafe] {
        driver::run(request_execution, seed, 6);
    }
}
