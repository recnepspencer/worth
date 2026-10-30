use worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence;

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<BridgeConditionalDecisionEvidence>();
}
