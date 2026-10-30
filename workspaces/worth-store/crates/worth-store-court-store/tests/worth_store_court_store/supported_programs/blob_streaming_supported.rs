#[path = "../sources/worth_store/physical_adapter_authority/blob_streaming_supported.rs"]
mod program;

#[test]
fn blob_streaming_supported_compiles_and_runs() {
    program::main();
}
