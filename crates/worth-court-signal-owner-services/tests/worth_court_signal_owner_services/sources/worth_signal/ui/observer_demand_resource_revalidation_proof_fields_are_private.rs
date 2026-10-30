use worth_signal::facade::core::ObserverDemandResourceRevalidationProof;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _proof = ObserverDemandResourceRevalidationProof {
        node: fake(),
        observer_id: 1,
        handle_id: 2,
        observation_digest: String::from("forged"),
        decision_digest: fake(),
    };
}
