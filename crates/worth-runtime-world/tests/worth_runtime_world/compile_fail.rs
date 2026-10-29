//! Compile fail integration courts.
#[allow(unused_imports)]
use crate::*;

#[test]
fn owned_async_progression_requires_actual_cleanup_custody() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/owned_async_progression_requires_custody.rs");
}

#[test]
fn compile_failures_protect_the_public_contract() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/pass/*.rs");
}
