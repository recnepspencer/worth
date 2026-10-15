#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;
use support::contracts;

#[test]
fn every_upfront_contract_door_is_compiler_enforced() {
    run_compile_fail_bundle(&contracts::compile_fail_bundle());
}

#[test]
fn the_upfront_contract_vocabulary_still_composes_on_the_public_lane() {
    run_compile_pass_bundle(&contracts::compile_pass_bundle());
}
