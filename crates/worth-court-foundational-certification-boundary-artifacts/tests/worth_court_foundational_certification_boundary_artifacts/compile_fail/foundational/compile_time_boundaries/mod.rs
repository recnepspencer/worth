#[allow(unused_imports)]
use crate::compile_time_boundaries::boundary_context::*;
#[test]
fn foundational_compile_time_boundaries_hold() {
    let tests = trybuild::TestCases::new();
    for glob in COMPILE_FAIL_GLOBS {
        tests.compile_fail(glob);
    }
    tests.pass(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../worth-testkit-foundational/assets/ui/expressions/pass/*.rs"
    ));
}
