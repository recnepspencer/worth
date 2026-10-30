use worth_runtime_bridge::facade::{BridgeSnapshotToken, TruthSnapshotIdentity};

fn main() {
    let _ = BridgeSnapshotToken::issued(TruthSnapshotIdentity::new("snapshot"), "token");
}
