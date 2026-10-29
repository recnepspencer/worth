// Declared case list for the up-front contract vocabulary suite.
//
// Declared rather than globbed on purpose. A glob passes when a case file is
// deleted or renamed, which is the one failure a compile-fail guard must never
// have: the guard needs a guard against finding nothing. Naming every case
// here means removing one is a suite failure, not a silent shrink.

use super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "upfront_contract_vocabulary",
        vec![
            CompileFailCase::new(
                "instance_identity",
                "tests/ui/contracts/compile_fail/brands_do_not_cross_scopes.rs",
            ),
            CompileFailCase::new(
                "linearity",
                "tests/ui/contracts/compile_fail/linear_resource_cannot_terminate_twice.rs",
            ),
            CompileFailCase::new(
                "evidence_minting",
                "tests/ui/contracts/compile_fail/freshness_sample_is_not_caller_mintable.rs",
            ),
            CompileFailCase::new(
                "evidence_minting",
                "tests/ui/contracts/compile_fail/causal_and_performed_evidence_are_not_caller_mintable.rs",
            ),
            CompileFailCase::new(
                "source_substitution",
                "tests/ui/contracts/compile_fail/freshness_source_and_policy_cannot_be_substituted.rs",
            ),
            CompileFailCase::new(
                "dynamic_value_authority",
                "tests/ui/contracts/compile_fail/binding_comparison_does_not_mint_reusable_authority.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "upfront_contract_vocabulary",
        vec![CompilePassCase::new(
            "composition",
            "tests/ui/contracts/compile_pass/upfront_contracts_compose.rs",
        )],
    )
}
