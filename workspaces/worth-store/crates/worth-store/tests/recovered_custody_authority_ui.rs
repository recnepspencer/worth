#![cfg_attr(test, allow(clippy::disallowed_methods))]
#[cfg(feature = "recovery-runtime-owner")]
#[test]
fn d7_continuous_lease_owner_moves_from_recovery_into_serving() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/recovered_custody_authority/d7_continuous_lease_owner_moves.rs");
    cases.compile_fail(
        "tests/recovered_custody_authority/d7_continuous_lease_cannot_be_detached.rs",
    );
}
