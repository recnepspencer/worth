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
        "lowering_and_execution_readiness_boundary",
        vec![
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "lowered_ready_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_fail/lowered_recipe_cannot_execute_without_readiness.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "pre_lowered_readiness_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_fail/resolved_recipe_cannot_enter_execution_readiness.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "bridged_lowered_readiness_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_fail/boundary_bridged_lowered_cannot_enter_execution_readiness.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "shifted_basis_ready_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_fail/shifted_basis_ready_recipe_cannot_be_treated_as_original_basis.rs",
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "lowering_and_execution_readiness_boundary",
        vec![
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "lowered_ready_executed_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_pass/explicit_lowered_ready_executed_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "checked_readiness_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_pass/checked_readiness_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "same_basis_runtime_readmission_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_pass/same_basis_runtime_readmission_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "shifted_basis_runtime_readmission_progression",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone5/compile_pass/shifted_basis_readiness_progression_compiles.rs",
            ),
        ],
    )
}
