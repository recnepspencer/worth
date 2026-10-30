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
        "transition_outcome_algebra",
        vec![
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "ordering_misuse",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/unresolved_recipe_cannot_lower_through_transition_contract.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "ordering_misuse",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/resolved_recipe_cannot_admit_through_transition_contract.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "ordering_misuse",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/resolved_recipe_cannot_enter_checked_resolution_pipeline.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "ordering_misuse",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/lowered_recipe_cannot_enter_checked_lowering_pipeline.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "ordering_misuse",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/resolved_recipe_cannot_enter_checked_admission_pipeline.rs",
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "transition_outcome_algebra",
        vec![
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "control_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/explicit_transition_contract_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "typed_outcome_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/typed_transition_outcomes_preserve_non_success_categories.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "checked_composition_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/checked_resolution_and_composition_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "freshness_failure_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/freshness_and_failure_checked_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "equivalent_admitted_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone4/equivalent_admitted_checked_progression_compiles.rs",
            ),
        ],
    )
}
