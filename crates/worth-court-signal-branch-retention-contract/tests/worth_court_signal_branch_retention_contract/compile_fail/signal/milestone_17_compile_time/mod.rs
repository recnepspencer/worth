#[test]
fn conditional_source_authority_cannot_be_forged_or_admitted_separately() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_17/arbitrary_marker_cannot_mint_source.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_branch_retention_contract/sources/worth_signal/ui/milestone_17/query_cannot_reach_signal_source_admission.rs")).expect("prebuilt compiler denial");
}
