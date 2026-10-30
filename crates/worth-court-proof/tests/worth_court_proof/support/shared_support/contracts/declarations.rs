// Declared case list for the up-front contract vocabulary suite.
//
// Declared rather than globbed on purpose. A glob passes when a case file is
// deleted or renamed, which is the one failure a compile-fail guard must never
// have: the guard needs a guard against finding nothing. Naming every case
// here means removing one is a suite failure, not a silent shrink.

pub(crate) use {
    crate::compile_fail::harness::bundle::CompileFailBundle,
    crate::compile_fail::harness::bundle::CompileFailCase,
};
pub(crate) use {
    crate::shared_support::compile_pass::bundle::CompilePassBundle,
    crate::shared_support::compile_pass::bundle::CompilePassCase,
};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::compile_fail::harness::bundle::CompileFailBundle::new(
        "upfront_contract_vocabulary",
        vec![
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "instance_identity",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/brands_do_not_cross_scopes.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "linearity",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/linear_resource_cannot_terminate_twice.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "evidence_minting",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/freshness_sample_is_not_caller_mintable.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "evidence_minting",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/causal_and_performed_evidence_are_not_caller_mintable.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "source_substitution",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/freshness_source_and_policy_cannot_be_substituted.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "dynamic_value_authority",
                "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_fail/binding_comparison_does_not_mint_reusable_authority.rs",
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "upfront_contract_vocabulary",
        vec![crate::shared_support::compile_pass::bundle::CompilePassCase::new(
            "composition",
            "tests/worth_court_proof/sources/worth_proof/ui/contracts/compile_pass/upfront_contracts_compose.rs",
        )],
    )
}
