use worth_runtime_bridge::facade::BridgeReplayMode;
use worth_runtime_bridge::facade::SnapshotReadPacket;

use worth_query_execution::facade::domain_computation::WorthQueryManagedTruthReadRequest;

#[test]
fn ordinary_managed_truth_requests_disable_replay_by_default() {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
    let identity = runtime.main_branch_identity();
    let (descriptor, _) = runtime.observe_branch(&identity).unwrap();
    let (_, _, replay, _, _, product_observation) =
        WorthQueryManagedTruthReadRequest::from_relational_basis_for_test(
            descriptor,
            SnapshotReadPacket::new(Vec::new()),
        )
        .into_parts_for_test();

    assert_eq!(replay, BridgeReplayMode::Disabled);
    assert!(product_observation.is_none());
}
