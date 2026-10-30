use worth_runtime_bridge::facade::BridgeRetainedConditionalDecisionSeed;

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<BridgeRetainedConditionalDecisionSeed>();
}
