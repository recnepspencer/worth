#[path = "../sources/worth_store/physical_adapter_authority/physical_integrity_supported.rs"]
mod program;

#[test]
fn physical_integrity_supported_compiles_and_runs() {
    program::main();
}
