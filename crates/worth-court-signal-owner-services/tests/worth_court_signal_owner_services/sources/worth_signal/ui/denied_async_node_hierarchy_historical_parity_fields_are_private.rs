use worth_signal::facade::core::DeniedAsyncNodeHierarchyHistoricalParity;

fn denial() -> DeniedAsyncNodeHierarchyHistoricalParity {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let denial = denial();
    let _ = denial.denial_class;
}
