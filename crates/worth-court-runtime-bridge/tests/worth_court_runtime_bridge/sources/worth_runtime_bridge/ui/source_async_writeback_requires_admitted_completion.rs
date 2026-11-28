use worth_foundational::facade::{AspectKey, AspectValue};
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionAdmissionReport, BridgeAsyncRequestTruthViewBasis,
    BridgeAsyncWritebackAdmissionRequest, BridgeWritebackEffectClass, BridgeWritebackEffectIntent,
    TruthBranchIdentity, TruthCommitIdentity, TruthSnapshotIdentity,
};

fn cannot_build_async_writeback_request_from_completion_report(
    report: &BridgeAsyncCompletionAdmissionReport,
) {
    let effect_intent = BridgeWritebackEffectIntent::validated_scalar_patch(
        BridgeWritebackEffectClass::ProjectedStateDiff,
        AspectKey::new("bridge.async.writeback").expect("static aspect key should validate"),
        AspectValue::String("value".to_owned().into()),
    )
    .expect("effect intent should validate");
    let current_truth_view_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        TruthBranchIdentity::from_bridge_harness_label("truth-main"),
        TruthCommitIdentity::from_bridge_harness_label("commit"),
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot"),
    );
    let _ = BridgeAsyncWritebackAdmissionRequest::authoritative_commit(
        report,
        effect_intent,
        current_truth_view_basis,
    );
}

fn main() {
    let _ = cannot_build_async_writeback_request_from_completion_report;
}
