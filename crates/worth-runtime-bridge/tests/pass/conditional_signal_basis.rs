use worth_runtime_bridge::facade::{
    BridgeConditionalEvaluationAdmissionRequest, BridgeConditionalSignalBasisBinding,
    BridgeSealedRuntimeAssembly, RelationalCommittedPatchRequest, TruthSnapshotIdentity,
};

fn admitted_operations(
    bridge: &BridgeSealedRuntimeAssembly,
    signal: &BridgeConditionalSignalBasisBinding,
    snapshot: &TruthSnapshotIdentity,
    patch: RelationalCommittedPatchRequest,
) {
    let _ = bridge.admit_conditional_evaluation(
        BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
            signal, snapshot,
        ),
    );
    let _ = bridge.deliver_authoritative_change(signal, 0, patch);
}

fn main() {}
