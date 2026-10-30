use worth_runtime_bridge::facade::BridgeMixedCauseDeliveryWindowPlan;

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let _ = BridgeMixedCauseDeliveryWindowPlan {
        delivery_window_identity: fake(),
        ordering_identity: fake(),
        delivery_family: fake(),
        ordered_causes: fake(),
        counters: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
