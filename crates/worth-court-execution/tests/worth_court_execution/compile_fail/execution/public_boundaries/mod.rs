#[test]
fn public_authority_boundaries() {
    let cases = trybuild::TestCases::new();
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/valid_map.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/forge_lease.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/forge_map.rs"));
}

#[test]
fn bounded_request_is_a_sealed_external_contract() {
    let cases = trybuild::TestCases::new();
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/serial_from_policy.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/serial_without_policy.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/scoped_request_lease.rs"));
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/escape_request_lease.rs"));
    cases.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/bounded_request.rs"));
}
