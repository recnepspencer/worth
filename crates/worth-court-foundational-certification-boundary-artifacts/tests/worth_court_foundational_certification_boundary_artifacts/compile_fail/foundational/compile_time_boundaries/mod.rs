#[allow(unused_imports)]
use crate::compile_time_boundaries::boundary_context::*;
#[test]
fn foundational_compile_time_boundaries_hold() {
    for glob in COMPILE_FAIL_GLOBS {
        worth_testkit_compiler::denied(glob).expect("prebuilt compiler denial");
    }
}
