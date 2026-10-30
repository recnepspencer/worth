#[path = "../sources/worth_store/physical_runtime_authority/protected_record_reads_supported.rs"]
mod program;

#[test]
fn protected_record_reads_supported_compiles_and_runs() {
    program::main();
}
