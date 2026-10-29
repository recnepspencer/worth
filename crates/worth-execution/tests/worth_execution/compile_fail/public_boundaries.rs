#[test]
fn public_authority_boundaries() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/valid_map.rs");
    cases.compile_fail("tests/ui/forge_lease.rs");
    cases.compile_fail("tests/ui/forge_map.rs");
}

#[test]
fn bounded_request_is_a_sealed_external_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/serial_from_policy.rs");
    cases.compile_fail("tests/ui/serial_without_policy.rs");
    cases.pass("tests/ui/scoped_request_lease.rs");
    cases.compile_fail("tests/ui/escape_request_lease.rs");
    cases.pass("tests/ui/bounded_request.rs");
}
