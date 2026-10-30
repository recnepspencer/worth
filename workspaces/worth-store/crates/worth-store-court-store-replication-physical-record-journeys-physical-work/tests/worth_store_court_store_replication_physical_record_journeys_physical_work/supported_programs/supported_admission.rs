#[path = "../sources/worth_store/physical_runtime_authority/supported_admission.rs"]
mod program;

#[test]
fn supported_admission_compiles_and_runs() {
    program::main();
}
