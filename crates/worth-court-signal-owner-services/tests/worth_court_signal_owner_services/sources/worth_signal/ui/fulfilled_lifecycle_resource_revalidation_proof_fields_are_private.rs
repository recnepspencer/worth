use worth_signal::facade::core::FulfilledLifecycleResourceRevalidationProof;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = FulfilledLifecycleResourceRevalidationProof {
        node: fake(),
        decision_digest: fake(),
    };
}
