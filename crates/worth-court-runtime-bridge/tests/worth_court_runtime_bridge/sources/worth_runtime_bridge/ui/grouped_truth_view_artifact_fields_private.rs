use worth_runtime_bridge::facade::{
    BridgeGroupedTruthViewArtifact, TruthSnapshotIdentity,
};

fn main() {
    let _ = BridgeGroupedTruthViewArtifact {
        truth_view_digest: sealed_authority_placeholder(),
        basis_snapshot_identity: TruthSnapshotIdentity::new("snapshot-a"),
        contract: sealed_authority_placeholder(),
        members: vec![],
        digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
