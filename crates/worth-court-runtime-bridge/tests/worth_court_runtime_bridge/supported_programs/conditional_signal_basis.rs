#[path = "../sources/worth_runtime_bridge/pass/conditional_signal_basis.rs"]
mod program;

#[test]
fn conditional_signal_basis_compiles_and_runs() {
    program::main();
}
