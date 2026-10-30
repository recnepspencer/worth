#[test]
fn physical_integrity_construction_boundaries() {
    let cases = trybuild::TestCases::new();
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/owner_valid_admission.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/family_substitution.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/proof_construction.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/root_tree_proof_construction.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/proof_escape.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/projection_cannot_open_decoder.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/record_cannot_open_decoder.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/scope_substitution.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/validated_view_cannot_open_decoder.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_physical_integrity/sources/worth_store_physical_integrity/ui/untrusted_page_extent_cannot_project.rs"));
}
