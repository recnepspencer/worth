use worth_runtime_bridge::facade::{BridgeWritebackFamilyKind, TruthWritebackRequest};

fn main() {
    let _request = TruthWritebackRequest {
        family_kind: BridgeWritebackFamilyKind::ProjectedStateDiff,
    };
}
