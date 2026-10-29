use worth_runtime_bridge::facade::{
    AdmittedBridgeSubscriptionResumeBasis, BridgeRetainedSubscriptionResumeBasis,
};

fn fake<T>() -> T {
    panic!("private")
}

fn main() {
    let _ = AdmittedBridgeSubscriptionResumeBasis {
        admitted_resume_basis_identity: fake(),
        retained_basis: fake::<BridgeRetainedSubscriptionResumeBasis>(),
        counters: fake(),
        canonical_basis: fake(),
        digest: fake(),
    };
}
