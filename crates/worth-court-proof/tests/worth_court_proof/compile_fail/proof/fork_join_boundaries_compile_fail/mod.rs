#[allow(unused_imports)]
use crate::shared_support as support;

use crate::compile_fail::harness::run_compile_fail_bundle;

#[test]
fn fixed_arity_fork_join_boundaries_hold() {
    run_compile_fail_bundle(&crate::shared_support::composition_family::compile_fail_bundle());
}
