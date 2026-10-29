use super::super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::super::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "transition_outcome_algebra",
        vec![
            CompileFailCase::new(
                "ordering_misuse",
                "tests/ui/milestone4/unresolved_recipe_cannot_lower_through_transition_contract.rs",
            ),
            CompileFailCase::new(
                "ordering_misuse",
                "tests/ui/milestone4/resolved_recipe_cannot_admit_through_transition_contract.rs",
            ),
            CompileFailCase::new(
                "ordering_misuse",
                "tests/ui/milestone4/resolved_recipe_cannot_enter_checked_resolution_pipeline.rs",
            ),
            CompileFailCase::new(
                "ordering_misuse",
                "tests/ui/milestone4/lowered_recipe_cannot_enter_checked_lowering_pipeline.rs",
            ),
            CompileFailCase::new(
                "ordering_misuse",
                "tests/ui/milestone4/resolved_recipe_cannot_enter_checked_admission_pipeline.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "transition_outcome_algebra",
        vec![
            CompilePassCase::new(
                "control_progression",
                "tests/ui/milestone4/explicit_transition_contract_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "typed_outcome_progression",
                "tests/ui/milestone4/typed_transition_outcomes_preserve_non_success_categories.rs",
            ),
            CompilePassCase::new(
                "checked_composition_progression",
                "tests/ui/milestone4/checked_resolution_and_composition_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "freshness_failure_progression",
                "tests/ui/milestone4/freshness_and_failure_checked_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "equivalent_admitted_progression",
                "tests/ui/milestone4/equivalent_admitted_checked_progression_compiles.rs",
            ),
        ],
    )
}
