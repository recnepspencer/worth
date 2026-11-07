use super::*;

#[test]
fn proof_shape_and_slope_surfaces_stay_exported() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let proof = worth_query_lower_runtime_proof_shape_audit();
    let surface = worth_query_lower_runtime_representative_surface(&resource_request)
        .expect("fixture digest must fit");
    let slopes = certify_lower_runtime_performance_slopes(&surface);

    assert_eq!(proof.rows().len(), 5);
    assert_eq!(slopes.rows().len(), 6);
    assert!(slopes
        .digest_for_output("debt_registry_lookup_slope_digest")
        .is_some());
    assert_eq!(slopes.profiles().len(), 3);
}
