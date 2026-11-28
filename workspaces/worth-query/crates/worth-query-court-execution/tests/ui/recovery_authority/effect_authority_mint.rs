use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;

fn fixture<T>() -> T {
    unreachable!()
}

fn main() {
    let _ = WorthQueryRecoveryEffectAuthority::mint_for_test(fixture(), fixture(), fixture());
}
