pub(crate) use {
    crate::shared_support::compile_pass::bundle::CompilePassBundle,
    crate::shared_support::compile_pass::bundle::CompilePassCase,
};

pub(crate) fn run_compile_pass_bundle(bundle: &CompilePassBundle) {
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
