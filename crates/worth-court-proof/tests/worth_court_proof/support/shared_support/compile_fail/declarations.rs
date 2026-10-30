pub(crate) use {
    crate::shared_support::compile_fail::bundle::CompileFailBundle,
    crate::shared_support::compile_fail::bundle::CompileFailCase,
};

pub(crate) fn run_compile_fail_bundle(bundle: &CompileFailBundle) {
    assert!(
        !bundle.cases().is_empty(),
        "compile-fail bundle for suite '{}' must not be empty",
        bundle.suite()
    );

    for case in bundle.cases() {
        worth_testkit_compiler::denied(case.path()).expect("prebuilt compiler denial");
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
