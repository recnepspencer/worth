use super::super::RuntimeCore;
use crate::expression::model::SignalValue;
use crate::recipe::model::SourceSpec;
use crate::runtime::compute_callbacks::ComputeCallbackInvocationResult;
use crate::runtime::policy::{RuntimePolicyPreset, RuntimePolicySpec};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[test]
fn demanded_callback_flow_export_respects_explicit_capture_policy() {
    for preset in [
        RuntimePolicyPreset::Forensic,
        RuntimePolicyPreset::Operational,
    ] {
        let retains_flow = preset == RuntimePolicyPreset::Forensic;
        let mut core = RuntimeCore::new(RuntimePolicySpec { preset }).unwrap();
        let invocations = Arc::new(AtomicUsize::new(0));
        let callback_invocations = Arc::clone(&invocations);
        core.define_source(SourceSpec {
            id: "input".into(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
        core.define_web_computed_native_callback(
            "derived".into(),
            Box::new(move || {
                callback_invocations.fetch_add(1, Ordering::Relaxed);
                Ok(ComputeCallbackInvocationResult {
                    value: SignalValue::Number(2.0),
                    captured_read_ids: vec!["input".into()],
                    captured_host_capability_reads: Vec::new(),
                    runtime_read_breadth: 1,
                    return_serialization_breadth: 1,
                })
            }),
        )
        .unwrap();
        assert_eq!(invocations.load(Ordering::Relaxed), 1);
        assert_eq!(
            core.read_value("derived").unwrap(),
            SignalValue::Number(2.0)
        );
        assert!(invocations.load(Ordering::Relaxed) > 1);
        let node = core.catalog.get("derived").unwrap().node;
        assert!(core
            .runtime
            .observe()
            .graph()
            .runtime_artifact_hot(node)
            .unwrap()
            .is_some());
        let exported = core.latest_flow().unwrap();
        if retains_flow {
            let exported = exported.expect("Forensic captures the demanded computation");
            let wire = serde_json::to_value(exported).unwrap();
            assert!(wire["flow"]["change"].is_object());
            assert!(wire["flow"]["apply"]["report"].is_object());
            assert!(wire["callbackNodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|node| {
                    node["id"] == "derived" && node["purityPosture"] == "signalTracked"
                }));
        } else {
            assert!(
                exported.is_none(),
                "Operational requires an explicit capture session"
            );
        }
    }
}
