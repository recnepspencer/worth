#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/causal_envelope/*.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/writeback_contract/*.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/relational_identity/*.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/crate_private_constructors/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn bridge_public_contracts_a_to_f_are_sealed() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[a-f]*.rs"
    ))
    .expect("prebuilt compiler denial");
}

#[test]
fn bridge_public_contracts_g_to_r_are_sealed() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[g-r]*.rs"
    ))
    .expect("prebuilt compiler denial");
}

#[test]
fn bridge_public_contracts_s_to_z_are_sealed() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[s-z]*.rs"
    ))
    .expect("prebuilt compiler denial");
}
