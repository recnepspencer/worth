#[test]
fn store_m4_compile_fail_fences() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
