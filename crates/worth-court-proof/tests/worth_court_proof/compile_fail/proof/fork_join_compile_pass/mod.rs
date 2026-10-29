#[allow(unused_imports)]
use {crate::shared_support as support};

use {crate::shared_support::compile_pass::run_compile_pass_bundle};
use {crate::shared_support::composition_family};

#[test]
fn fixed_arity_fork_join_representative_progression_compiles() {
    run_compile_pass_bundle(&crate::shared_support::composition_family::compile_pass_bundle());
}
