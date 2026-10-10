use worth_runtime_bridge::facade::{
    BridgeConditionalEvaluationAdmissionRequest, BridgeInstalledConditionalLowering,
    BridgeSealedRuntimeAssembly, RelationalCommittedPatchRequest, TruthSnapshotIdentity,
};

fn lowering_cannot_select_signal_basis(
    bridge: &BridgeSealedRuntimeAssembly,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    lowering: &BridgeInstalledConditionalLowering,
    snapshot: &TruthSnapshotIdentity,
    patch: RelationalCommittedPatchRequest,
) {
    let _ = BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
        lowering, snapshot,
    );
    let _ = bridge.deliver_authoritative_change(execution, lowering, 0, patch);
}

fn main() {}
