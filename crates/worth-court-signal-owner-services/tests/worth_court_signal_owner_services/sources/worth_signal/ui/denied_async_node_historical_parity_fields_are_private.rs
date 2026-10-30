use worth_signal::facade::core::DeniedAsyncNodeHistoricalParity;

fn denied_parity() -> DeniedAsyncNodeHistoricalParity {
    panic!("private-field compile-fail fixture")
}

fn main() {
    let denial = denied_parity();
    let _ = denial.denial_class;
}
