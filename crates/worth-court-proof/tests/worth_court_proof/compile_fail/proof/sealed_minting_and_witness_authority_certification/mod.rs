#[allow(unused_imports)]
use {crate::shared_support as support};

use {crate::shared_support::compile_fail::run_compile_fail_bundle};
use {crate::shared_support::sealed_minting};

#[test]
fn sealed_minting_and_witness_authority_certification() {
    crate::shared_support::sealed_minting::assert_fixture_completeness();
    let codegen_honesty_report = crate::shared_support::sealed_minting::codegen_honesty_report();

    run_compile_fail_bundle(&crate::shared_support::sealed_minting::compile_fail_bundle());

    assert!(codegen_honesty_report
        .checks()
        .iter()
        .all(|check| check.matches()));
    assert!(!codegen_honesty_report.hidden_dynamic_lookup());
    assert!(!codegen_honesty_report.hidden_virtual_dispatch());
    assert!(!codegen_honesty_report.mandatory_allocation_introduced());
}
