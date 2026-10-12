fn prepare_compiler_target() {
    let workspace_temp = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("trybuild");
    std::fs::create_dir_all(&workspace_temp).expect("trybuild temp directory");
    std::env::set_var("HOME", &workspace_temp);
    std::env::set_var("USERPROFILE", &workspace_temp);
    std::env::set_var("TMP", &workspace_temp);
    std::env::set_var("TEMP", &workspace_temp);
    std::env::set_var("CARGO_TARGET_DIR", workspace_temp.join("cargo-target"));
}

#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    prepare_compiler_target();
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/conditional_signal_basis.rs");
    t.compile_fail("tests/ui/causal_envelope/*.rs");
    t.compile_fail("tests/ui/writeback_contract/*.rs");
    t.compile_fail("tests/ui/relational_identity/*.rs");
    t.compile_fail("tests/ui/crate_private_constructors/*.rs");
}

#[test]
fn bridge_public_contracts_a_to_f_are_sealed() {
    prepare_compiler_target();
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/[a-f]*.rs");
}

#[test]
fn bridge_public_contracts_g_to_r_are_sealed() {
    prepare_compiler_target();
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/[g-r]*.rs");
}

#[test]
fn bridge_public_contracts_s_to_z_are_sealed() {
    prepare_compiler_target();
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/[s-z]*.rs");
}
