use worth_signal::facade::core::ActiveResourceRevalidationProof;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = ActiveResourceRevalidationProof {
        node: fake(),
        handle: fake(),
        decision_digest: fake(),
    };
}
