#[path = "../sources/worth_proof/ui/milestone6/compile_pass/checked_multi_input_ordering_and_ready_join_compiles.rs"]
mod program_1;

#[path = "../sources/worth_proof/ui/milestone6/compile_pass/explicit_family_symbol_resolution_and_lowering_compiles.rs"]
mod program_2;

#[path = "../sources/worth_proof/ui/milestone6/compile_pass/explicit_fixed_arity_fork_join_progression_compiles.rs"]
mod program_3;

#[test]
fn fixed_arity_fork_join_representative_progression_compiles() {
    program_1::main();
    program_2::main();
    program_3::main();
}
