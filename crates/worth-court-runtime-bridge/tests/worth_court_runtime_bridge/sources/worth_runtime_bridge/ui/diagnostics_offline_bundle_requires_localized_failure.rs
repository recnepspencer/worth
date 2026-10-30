use worth_runtime_bridge::facade::BridgeTemporalAsyncOfflineDiagnosisBundleDraft;

fn main() {
    let _ = BridgeTemporalAsyncOfflineDiagnosisBundleDraft::new(vec![String::from(
        "raw diagnostics strings are not localized failures",
    )]);
}
