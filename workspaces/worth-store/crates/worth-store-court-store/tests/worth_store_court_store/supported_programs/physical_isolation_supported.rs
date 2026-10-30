#[path = "../sources/worth_store/physical_adapter_authority/physical_isolation_supported.rs"]
mod program;

#[test]
fn physical_isolation_supported_compiles_and_runs() {
    program::main();
}
