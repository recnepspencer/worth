#[path = "../sources/worth_store/physical_runtime_authority/blob_ingest_supported.rs"]
mod program;

#[test]
fn blob_ingest_supported_compiles_and_runs() {
    program::main();
}
