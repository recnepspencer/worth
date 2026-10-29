#[allow(unused_imports)]
use {crate::shared_support as support};

use {crate::shared_support::compile_pass::run_compile_pass_bundle};
use {crate::shared_support::dx};

#[test]
fn pleasant_entrypoints_remain_additive_guidance() {
    run_compile_pass_bundle(&crate::shared_support::dx::compile_pass_bundle());
}
