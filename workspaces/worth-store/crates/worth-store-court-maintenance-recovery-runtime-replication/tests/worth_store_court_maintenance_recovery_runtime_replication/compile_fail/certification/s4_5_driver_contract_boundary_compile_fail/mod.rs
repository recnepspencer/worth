#[allow(unused_imports)]
use crate::s4_5_driver_contract_boundary_compile_fail::*;

#[test]
fn driver_contract_boundary_rejects_arbitrary_named_yieldpoint_authority() {
    run_fixture(
        "named_yieldpoint_authority_cannot_be_minted.rs",
        &["named_production_boundary"],
    );
}
