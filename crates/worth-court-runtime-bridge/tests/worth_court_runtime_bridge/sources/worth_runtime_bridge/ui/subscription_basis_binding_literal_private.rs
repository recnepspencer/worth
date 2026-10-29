use worth_runtime_bridge::facade::{
    BridgeSubscriptionBasisKind, BridgeSubscriptionBasisRequest, ValidatedSubscriptionBasisBinding,
    TruthSnapshotIdentity,
};

fn main() {
    let _request = BridgeSubscriptionBasisRequest::snapshot(TruthSnapshotIdentity::new(
        "snapshot-a",
    ));

    let _ = ValidatedSubscriptionBasisBinding {
        basis_kind: BridgeSubscriptionBasisKind::Snapshot,
    };
}
