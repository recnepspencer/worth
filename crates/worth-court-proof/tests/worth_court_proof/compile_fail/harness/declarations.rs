pub(crate) use crate::compile_fail::harness::bundle::CompileFailBundle;

pub(crate) fn run_compile_fail_bundle(bundle: &CompileFailBundle) {
    assert!(
        !bundle.cases().is_empty(),
        "compile-fail bundle for suite '{}' must not be empty",
        bundle.suite()
    );
    let cases = trybuild::TestCases::new();
    for case in bundle.cases() {
        cases.compile_fail(case.path());
    }
}

pub(crate) fn run_compile_fail_family(bundle: &CompileFailBundle, family: &'static str) {
    assert!(
        bundle.contains_family(family),
        "compile-fail family '{family}' is not declared for suite '{}'",
        bundle.suite()
    );
    run_compile_fail_bundle(&bundle.cases_for_family(family));
}

pub(crate) fn run_compile_fail_families(bundle: &CompileFailBundle, families: &[&'static str]) {
    for family in families {
        assert!(
            bundle.contains_family(family),
            "compile-fail family '{family}' is not declared for suite '{}'",
            bundle.suite()
        );
    }
    run_compile_fail_bundle(&bundle.cases_for_families(families));
}
