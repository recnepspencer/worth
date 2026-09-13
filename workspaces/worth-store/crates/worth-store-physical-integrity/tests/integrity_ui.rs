#[test]
fn physical_integrity_construction_boundaries() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/owner_valid_admission.rs");
    cases.compile_fail("tests/ui/family_substitution.rs");
    cases.compile_fail("tests/ui/proof_construction.rs");
    cases.compile_fail("tests/ui/root_tree_proof_construction.rs");
    cases.compile_fail("tests/ui/proof_escape.rs");
    cases.compile_fail("tests/ui/projection_cannot_open_decoder.rs");
    cases.compile_fail("tests/ui/record_cannot_open_decoder.rs");
    cases.compile_fail("tests/ui/scope_substitution.rs");
    cases.compile_fail("tests/ui/validated_view_cannot_open_decoder.rs");
    cases.compile_fail("tests/ui/untrusted_page_extent_cannot_project.rs");
}
