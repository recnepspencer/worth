#[path = "../sources/worth_store/physical_runtime_authority/blob_relocation_supported.rs"]
mod program;

#[test]
fn blob_relocation_supported_compiles_and_runs() {
    program::main();
}
