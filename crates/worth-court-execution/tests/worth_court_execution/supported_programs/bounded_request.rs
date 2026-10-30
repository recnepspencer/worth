#[path = "../sources/worth_execution/ui/bounded_request.rs"]
mod program;

#[test]
fn bounded_request_compiles_and_runs() {
    program::main();
}
