#[test]
fn every_resident_field_requires_its_owner_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass(
        "tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/resident_forms.rs",
    );
    cases.compile_fail(
        "tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/missing_owner.rs",
    );
    cases.compile_fail("tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/unsupported_options.rs");
    cases.compile_fail(
        "tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/unsupported_items.rs",
    );
}
