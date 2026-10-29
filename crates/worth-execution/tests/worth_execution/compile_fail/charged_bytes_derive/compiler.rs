#[test]
fn every_resident_field_requires_its_owner_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/charged_bytes/resident_forms.rs");
    cases.compile_fail("tests/ui/charged_bytes/missing_owner.rs");
    cases.compile_fail("tests/ui/charged_bytes/unsupported_options.rs");
    cases.compile_fail("tests/ui/charged_bytes/unsupported_items.rs");
}
