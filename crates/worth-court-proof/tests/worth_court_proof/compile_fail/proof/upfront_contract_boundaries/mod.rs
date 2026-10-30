#[allow(unused_imports)]
use crate::shared_support as support;

use crate::shared_support::compile_fail::run_compile_fail_bundle;

use crate::shared_support::contracts;

#[test]
fn every_upfront_contract_door_is_compiler_enforced() {
    run_compile_fail_bundle(&crate::shared_support::contracts::compile_fail_bundle());
}
