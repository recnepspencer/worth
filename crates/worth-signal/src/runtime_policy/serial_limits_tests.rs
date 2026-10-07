use super::*;

#[test]
fn checkpoint_authority_restores_the_head_policy_shape() {
    let graph = crate::data::graph::SignalGraph::new();
    let mut wire = serde_json::to_value(graph.capture_checkpoint_authority()).unwrap();
    wire["installed_policy"]["requested_policy"]
        .as_object_mut()
        .unwrap()
        .remove("serial_memory_bytes");
    wire["diagnostics"]["request_mirror"]
        .as_object_mut()
        .unwrap()
        .remove("serial_memory_bytes");
    let authority = serde_json::from_value(wire).unwrap();
    let restored =
        crate::data::graph::SignalGraph::restore_from_checkpoint_authority(&authority).unwrap();
    assert_eq!(
        restored.runtime_policy(),
        SignalRuntimePolicy::development()
    );
}

#[test]
fn diagnostics_mirror_restores_missing_memory_without_defaulting_existing_policy() {
    let requested =
        SignalRuntimePolicy::for_tier(crate::diagnostics::profile::DiagnosticsTier::Forensic);
    let mut diagnostics = crate::diagnostics::state::DiagnosticsState::default();
    diagnostics.set_request_mirror(requested);
    let mut wire = serde_json::to_value(diagnostics).unwrap();
    wire["request_mirror"]
        .as_object_mut()
        .unwrap()
        .remove("serial_memory_bytes");
    let restored: crate::diagnostics::state::DiagnosticsState =
        serde_json::from_value(wire).unwrap();
    let restored_wire = serde_json::to_value(restored).unwrap();
    let restored_policy: SignalRuntimePolicy =
        serde_json::from_value(restored_wire["request_mirror"].clone()).unwrap();
    assert_eq!(restored_policy, requested);
    assert_eq!(restored_policy.serial_memory_bytes, 64 << 20);
}
