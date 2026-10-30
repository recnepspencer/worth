use worth_signal::facade::core::DeniedAsyncKeyedNodeHistoricalParity;

fn denial() -> DeniedAsyncKeyedNodeHistoricalParity {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let denial = denial();
    let _ = denial.denial_class;
}
