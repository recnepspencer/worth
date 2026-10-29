#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_pass::run_compile_pass_bundle;
use support::dx;

#[test]
fn pleasant_entrypoints_remain_additive_guidance() {
    run_compile_pass_bundle(&dx::compile_pass_bundle());
}
