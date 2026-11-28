use worth_runtime_bridge::facade::{BridgeSnapshotToken, TruthSnapshotIdentity};

fn main() {
    let _ = BridgeSnapshotToken::issued(
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot"),
        "token",
    );
}
