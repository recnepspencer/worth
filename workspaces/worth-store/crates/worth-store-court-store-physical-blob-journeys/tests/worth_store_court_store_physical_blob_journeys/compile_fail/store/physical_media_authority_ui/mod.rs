#[test]
fn runtime_progression_is_compiler_sealed_and_public_raw_facades_stay_private() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/borrowed_record_observation_cannot_promote_recovery.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/borrowed_wal_observation_cannot_promote_recovery.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/media_runtime_authority_is_sealed.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/non_authority_values_cannot_promote.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/raw_media_surface_is_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/optional_capabilities_require_handles.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_store_court_store_physical_blob_journeys/sources/worth_store/physical_media_authority/maximal_features_cannot_mint_authority.rs")).expect("prebuilt compiler denial");
}
