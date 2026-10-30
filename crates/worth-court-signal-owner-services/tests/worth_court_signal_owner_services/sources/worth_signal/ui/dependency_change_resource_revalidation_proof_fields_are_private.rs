use worth_signal::facade::core::DependencyChangeResourceRevalidationProof;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = DependencyChangeResourceRevalidationProof {
        node: fake(),
        node_state: fake(),
        decision_digest: fake(),
    };
}
