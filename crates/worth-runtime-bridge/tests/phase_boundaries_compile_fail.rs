#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    let workspace_temp = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("trybuild");
    std::fs::create_dir_all(&workspace_temp).expect("trybuild temp directory");
    std::env::set_var("HOME", &workspace_temp);
    std::env::set_var("USERPROFILE", &workspace_temp);
    std::env::set_var("TMP", &workspace_temp);
    std::env::set_var("TEMP", &workspace_temp);
    std::env::set_var("CARGO_TARGET_DIR", workspace_temp.join("cargo-target"));

    let t = trybuild::TestCases::new();
    t.pass("tests/pass/conditional_signal_basis.rs");
    t.compile_fail("tests/ui/causal_envelope/*.rs");
    t.compile_fail("tests/ui/writeback_contract/*.rs");
    t.compile_fail("tests/ui/relational_identity/*.rs");
    t.compile_fail("tests/ui/crate_private_constructors/*.rs");
    #[cfg(not(feature = "certification-construction"))]
    t.compile_fail("tests/ui/certification_constructors/*.rs");
    #[cfg(feature = "certification-construction")]
    t.pass("tests/pass/certification_constructors_with_feature.rs");
    t.compile_fail("tests/ui/*.rs");
}
