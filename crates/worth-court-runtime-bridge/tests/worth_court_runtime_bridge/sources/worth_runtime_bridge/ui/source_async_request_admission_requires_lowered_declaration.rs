use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, RuntimeBridge, TruthBranchIdentity, TruthCommitIdentity,
    TruthSnapshotIdentity, ValidatedBridgeAsyncSourceDeclaration,
};

fn fake<T>() -> T {
    panic!("type-only")
}

fn main() {
    let runtime: RuntimeBridge = fake();
    let validated: ValidatedBridgeAsyncSourceDeclaration = fake();
    let basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        TruthBranchIdentity::from_bridge_harness_label("truth-main"),
        TruthCommitIdentity::from_bridge_harness_label("commit-a"),
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
    );
    let _ = runtime.bind_async_request_basis(&validated, basis);
}
