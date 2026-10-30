use worth_runtime_bridge::facade::{
    BridgeTemporalAsyncFailureLocalizationMatrix, BridgeTemporalAsyncOfflineDiagnosisBundleDraft,
};

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let draft: BridgeTemporalAsyncOfflineDiagnosisBundleDraft = fake();
    let _ = BridgeTemporalAsyncFailureLocalizationMatrix::from_bundle(&draft);
}
