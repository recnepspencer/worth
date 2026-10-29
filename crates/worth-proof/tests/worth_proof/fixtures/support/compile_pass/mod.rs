#[path = "bundle.rs"]
pub(crate) mod bundle;

pub use bundle::{CompilePassBundle, CompilePassCase};

pub fn run_compile_pass_bundle(bundle: &CompilePassBundle) {
    assert!(
        !bundle.cases().is_empty(),
        "compile-pass bundle for suite '{}' must not be empty",
        bundle.suite()
    );
    let cases = trybuild::TestCases::new();
    for case in bundle.cases() {
        cases.pass(case.path());
    }
}
