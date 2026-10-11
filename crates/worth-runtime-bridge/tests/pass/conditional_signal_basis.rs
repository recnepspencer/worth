use worth_runtime_bridge::facade::{
    BridgeConditionalEvaluationAdmissionRequest, BridgeConditionalSignalBasisBinding,
    BridgeSealedRuntimeAssembly, RelationalCommittedPatchRequest, TruthSnapshotIdentity,
};

fn admitted_operations(
    bridge: &BridgeSealedRuntimeAssembly,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    signal: &BridgeConditionalSignalBasisBinding,
    snapshot: &TruthSnapshotIdentity,
    patch: RelationalCommittedPatchRequest,
) {
    let _ = bridge.admit_conditional_evaluation(
        BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
            signal, snapshot,
        ),
        execution,
    );
    let _ = bridge.deliver_authoritative_change(execution, signal, 0, patch);
}

fn main() {}
