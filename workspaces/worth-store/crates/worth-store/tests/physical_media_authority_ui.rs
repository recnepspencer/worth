#[test]
fn runtime_progression_is_compiler_sealed_and_public_raw_facades_stay_private() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/physical_media_authority/supported_media_admission.rs");
    cases.compile_fail("tests/physical_media_authority/media_runtime_authority_is_sealed.rs");
    cases.compile_fail("tests/physical_media_authority/non_authority_values_cannot_promote.rs");
    cases.compile_fail("tests/physical_media_authority/raw_media_surface_is_private.rs");
    cases.compile_fail("tests/physical_media_authority/optional_capabilities_require_handles.rs");
    cases.compile_fail("tests/physical_media_authority/maximal_features_cannot_mint_authority.rs");
}
