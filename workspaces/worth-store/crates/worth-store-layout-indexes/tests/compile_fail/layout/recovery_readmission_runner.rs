use super::compile_fail_support;

#[test]
fn recovery_imports_cannot_mint_readmission_authority() {
    compile_fail_support::assert_compile_fails_in_ui_dir(
        "foundations",
        "recovery_import_shortcut_witness_surface_is_not_public.rs",
        &["private field", "PhysicalSourceSelection"],
        &["worth_store_recovery_physics"],
    );
}
