#[allow(unused_imports)] use crate::s4_5_driver_contract_boundary_compile_fail::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn driver_contract_boundary_rejects_arbitrary_named_yieldpoint_authority() {
    run_fixture(
        "named_yieldpoint_authority_cannot_be_minted.rs",
        &["named_production_boundary"],
    );
}
