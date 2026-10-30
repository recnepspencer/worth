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
        TruthBranchIdentity::new("truth-main"),
        TruthCommitIdentity::new("commit-a"),
        TruthSnapshotIdentity::new("snapshot-a"),
    );
    let _ = runtime.bind_async_request_basis(&validated, basis);
}
