use worth_signal::facade::core::TerminalStateResourceRevalidationProof;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = TerminalStateResourceRevalidationProof {
        node: fake(),
        lifecycle: fake(),
        decision_digest: fake(),
    };
}
