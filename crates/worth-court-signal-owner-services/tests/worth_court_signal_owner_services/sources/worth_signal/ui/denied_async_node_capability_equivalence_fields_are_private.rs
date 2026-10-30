use worth_signal::facade::core::DeniedAsyncNodeCapabilityEquivalence;

fn denied_equivalence() -> DeniedAsyncNodeCapabilityEquivalence {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let denial = denied_equivalence();
    let _ = denial.denial_class;
}
