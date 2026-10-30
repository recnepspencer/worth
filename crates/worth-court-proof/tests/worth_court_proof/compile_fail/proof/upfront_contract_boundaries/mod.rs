#[allow(unused_imports)]
use crate::shared_support as support;

use crate::compile_fail::harness::run_compile_fail_bundle;

#[test]
fn every_upfront_contract_door_is_compiler_enforced() {
    run_compile_fail_bundle(&crate::shared_support::contracts::compile_fail_bundle());
}
