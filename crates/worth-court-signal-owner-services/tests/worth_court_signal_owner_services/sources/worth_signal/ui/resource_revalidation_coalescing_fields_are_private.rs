use worth_signal::facade::core::ResourceRevalidationCoalescing;

fn fake<T>() -> T {
    panic!("compile-fail fixture")
}

fn main() {
    let _coalescing = ResourceRevalidationCoalescing {
        winner: fake(),
        coalesced_request: fake(),
        freshness_decision: fake(),
        lifecycle_transition: fake(),
    };
}
