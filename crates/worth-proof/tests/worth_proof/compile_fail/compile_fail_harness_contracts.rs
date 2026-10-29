#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::{run_compile_fail_bundle, run_compile_fail_family, CompileFailBundle};
use support::core_artifact;

#[test]
#[should_panic(expected = "compile-fail family 'missing_family' is not declared")]
fn compile_fail_family_selection_fails_closed_for_unknown_family() {
    let bundle = core_artifact::compile_fail_bundle();
    run_compile_fail_family(&bundle, "missing_family");
}

#[test]
#[should_panic(expected = "compile-fail bundle for suite 'empty_suite' must not be empty")]
fn compile_fail_bundle_runner_rejects_empty_bundles() {
    let empty_bundle = CompileFailBundle::new("empty_suite", Vec::new());
    run_compile_fail_bundle(&empty_bundle);
}
