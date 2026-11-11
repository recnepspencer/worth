#[test]
fn continuous_lease_owner_moves_from_recovery_into_serving() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store/sources/worth_store/recovered_custody_authority/d7_continuous_lease_owner_moves.rs")).expect("prebuilt compiler denial");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store/sources/worth_store/recovered_custody_authority/d7_continuous_lease_cannot_be_detached.rs");
    for obligation in [
        "fields `media` and `custody` of struct `RecoveredPhysicalServingHandoff` are private",
        "fields `media` and `custody` of struct `RecoveredMediaOwnedPhysicalRuntime` are private",
    ] {
        worth_testkit_compiler::certification_semantic_denial(path, "E0451", &[obligation])
            .expect("each actual owner keeps its media and custody coupled");
    }
}
