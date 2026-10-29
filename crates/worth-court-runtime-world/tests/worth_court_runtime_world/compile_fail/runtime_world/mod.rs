//! Compile fail integration courts.
#[allow(unused_imports)]
use {};

#[test]
fn owned_async_progression_requires_actual_cleanup_custody() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_world/sources/worth_runtime_world/ui/owned_async_progression_requires_custody.rs"));
}

#[test]
fn compile_failures_protect_the_public_contract() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_world/sources/worth_runtime_world/ui/*.rs"));
    tests.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_world/sources/worth_runtime_world/pass/*.rs"));
}
