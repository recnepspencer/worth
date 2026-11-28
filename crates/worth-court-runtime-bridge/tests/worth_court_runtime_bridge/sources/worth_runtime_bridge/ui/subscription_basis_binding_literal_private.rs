use worth_runtime_bridge::facade::{
    BridgeSubscriptionBasisKind, BridgeSubscriptionBasisRequest, TruthSnapshotIdentity,
    ValidatedSubscriptionBasisBinding,
};

fn main() {
    let _request = BridgeSubscriptionBasisRequest::snapshot(
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
    );

    let _ = ValidatedSubscriptionBasisBinding {
        basis_kind: BridgeSubscriptionBasisKind::Snapshot,
    };
}
