use worth_runtime_bridge::facade::{
    BridgeSubscriptionReferenceWorkloadDeclaration, BridgeSubscriptionReferenceWorkloadLaneArtifactSet,
    BridgeSubscriptionReferenceWorkloadManifestSealed, BridgeSubscriptionReferenceWorkloadReport,
    RuntimeBridge,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let manifest: BridgeSubscriptionReferenceWorkloadManifestSealed = fake();
    let declaration: BridgeSubscriptionReferenceWorkloadDeclaration = fake();
    let lane_artifacts: BridgeSubscriptionReferenceWorkloadLaneArtifactSet = fake();
    let report: BridgeSubscriptionReferenceWorkloadReport = fake();

    let _ = runtime.seal_subscription_reference_workload_sufficiency(
        &manifest,
        &declaration,
        lane_artifacts,
        &report,
    );
}
