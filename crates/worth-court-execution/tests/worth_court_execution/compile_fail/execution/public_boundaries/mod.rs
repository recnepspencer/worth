#[test]
fn public_authority_boundaries() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_execution/sources/worth_execution/ui/forge_lease.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_execution/sources/worth_execution/ui/forge_map.rs"
    ))
    .expect("prebuilt compiler denial");
}

#[test]
fn bounded_request_is_a_sealed_external_contract() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_execution/sources/worth_execution/ui/serial_without_policy.rs"
    ))
    .expect("prebuilt compiler denial");

    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_execution/sources/worth_execution/ui/escape_request_lease.rs"
    ))
    .expect("prebuilt compiler denial");
}
