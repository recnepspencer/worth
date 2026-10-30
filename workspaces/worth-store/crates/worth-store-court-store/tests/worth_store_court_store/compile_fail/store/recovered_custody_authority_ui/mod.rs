#[test]
fn continuous_lease_owner_moves_from_recovery_into_serving() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store/sources/worth_store/recovered_custody_authority/d7_continuous_lease_owner_moves.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store/sources/worth_store/recovered_custody_authority/d7_continuous_lease_cannot_be_detached.rs")).expect("prebuilt compiler denial");
}
