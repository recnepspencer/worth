#[path = "../sources/worth_runtime_world/pass/required_execution_policy.rs"]
mod program;

#[test]
fn required_execution_policy_compiles_and_runs() {
    program::main();
}
