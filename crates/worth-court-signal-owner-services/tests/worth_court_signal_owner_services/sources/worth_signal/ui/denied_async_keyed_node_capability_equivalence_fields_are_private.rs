use worth_signal::facade::core::DeniedAsyncKeyedNodeCapabilityEquivalence;

fn denial() -> DeniedAsyncKeyedNodeCapabilityEquivalence {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let denial = denial();
    let _ = denial.denial_class;
}
