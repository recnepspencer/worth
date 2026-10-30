use worth_runtime_bridge::facade::BridgeSharedConsumerDeliveryBundleSealed;

fn fake<T>() -> T {
    panic!("fixture should never run")
}

fn main() {
    let _ = BridgeSharedConsumerDeliveryBundleSealed {
        shared_delivery_bundle_sealed_identity: fake(),
        layout_identity: fake(),
        active_subscription_identity: fake(),
        admitted_subscription_identity: fake(),
        mixed_cause_delivery_window_identity: fake(),
        fanout_layout_identity: fake(),
        delivery_family_identity: fake(),
        ordered_causes: fake(),
        consumer_contract_identities: fake(),
        counters: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
