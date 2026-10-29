#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    let t = trybuild::TestCases::new();
    t.pass(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/pass/conditional_signal_basis.rs"));
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/causal_envelope/*.rs"));
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/writeback_contract/*.rs"));
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/relational_identity/*.rs"));
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/crate_private_constructors/*.rs"));
}

#[test]
fn bridge_public_contracts_a_to_f_are_sealed() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[a-f]*.rs"));
}

#[test]
fn bridge_public_contracts_g_to_r_are_sealed() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[g-r]*.rs"));
}

#[test]
fn bridge_public_contracts_s_to_z_are_sealed() {
    let t = trybuild::TestCases::new();
    t.compile_fail(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[s-z]*.rs"));
}
