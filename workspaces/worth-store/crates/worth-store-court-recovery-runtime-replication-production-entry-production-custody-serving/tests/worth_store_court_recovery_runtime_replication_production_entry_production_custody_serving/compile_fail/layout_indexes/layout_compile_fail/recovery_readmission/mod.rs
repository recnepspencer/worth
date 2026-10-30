use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

#[test]
fn recovery_imports_cannot_mint_readmission_authority() {
    crate::compile_fail_support::assert_compile_fails_in_ui_dir(
        UiCompilerEnvironment::Production,
        "foundations",
        "recovery_import_shortcut_witness_surface_is_not_public.rs",
        &["private field", "PhysicalSourceSelection"],
    );
}
