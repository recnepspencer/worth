use worth_runtime_bridge::facade::BridgeWritebackFamilyKind;

const ACKNOWLEDGED_WRITEBACK_FAMILY_COUNT: usize = 1;

fn main() {
    let _acknowledged_families: [BridgeWritebackFamilyKind; ACKNOWLEDGED_WRITEBACK_FAMILY_COUNT] = [
        BridgeWritebackFamilyKind::ProjectedStateDiff,
        BridgeWritebackFamilyKind::AspectReconciliation,
    ];
}
