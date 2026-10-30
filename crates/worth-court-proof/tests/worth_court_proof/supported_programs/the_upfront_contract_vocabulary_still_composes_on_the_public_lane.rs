#[path = "../sources/worth_proof/ui/contracts/compile_pass/upfront_contracts_compose.rs"]
mod program;

#[test]
fn the_upfront_contract_vocabulary_still_composes_on_the_public_lane() {
    program::main();
}
