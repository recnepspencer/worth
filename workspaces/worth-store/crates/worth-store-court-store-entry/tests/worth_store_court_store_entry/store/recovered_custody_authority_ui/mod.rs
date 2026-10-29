#[cfg(all())]
#[test]
fn d7_continuous_lease_owner_moves_from_recovery_into_serving() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_entry/sources/worth_store/recovered_custody_authority/d7_continuous_lease_owner_moves.rs"));
    cases.compile_fail(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_entry/sources/worth_store/recovered_custody_authority/d7_continuous_lease_cannot_be_detached.rs"),
    );
}
