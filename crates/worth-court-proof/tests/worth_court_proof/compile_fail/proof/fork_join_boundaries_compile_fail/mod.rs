#[allow(unused_imports)]
use {crate::shared_support as support};

use {crate::shared_support::compile_fail::run_compile_fail_bundle};
use {crate::shared_support::composition_family};

#[test]
fn fixed_arity_fork_join_boundaries_hold() {
    run_compile_fail_bundle(&crate::shared_support::composition_family::compile_fail_bundle());
}
