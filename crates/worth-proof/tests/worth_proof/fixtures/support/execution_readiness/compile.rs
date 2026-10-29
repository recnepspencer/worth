use super::super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::super::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "lowering_and_execution_readiness_boundary",
        vec![
            CompileFailCase::new(
                "lowered_ready_boundary",
                "tests/ui/milestone5/compile_fail/lowered_recipe_cannot_execute_without_readiness.rs",
            ),
            CompileFailCase::new(
                "pre_lowered_readiness_boundary",
                "tests/ui/milestone5/compile_fail/resolved_recipe_cannot_enter_execution_readiness.rs",
            ),
            CompileFailCase::new(
                "bridged_lowered_readiness_boundary",
                "tests/ui/milestone5/compile_fail/boundary_bridged_lowered_cannot_enter_execution_readiness.rs",
            ),
            CompileFailCase::new(
                "shifted_basis_ready_boundary",
                "tests/ui/milestone5/compile_fail/shifted_basis_ready_recipe_cannot_be_treated_as_original_basis.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "lowering_and_execution_readiness_boundary",
        vec![
            CompilePassCase::new(
                "lowered_ready_executed_progression",
                "tests/ui/milestone5/compile_pass/explicit_lowered_ready_executed_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "checked_readiness_progression",
                "tests/ui/milestone5/compile_pass/checked_readiness_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "same_basis_runtime_readmission_progression",
                "tests/ui/milestone5/compile_pass/same_basis_runtime_readmission_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "shifted_basis_runtime_readmission_progression",
                "tests/ui/milestone5/compile_pass/shifted_basis_readiness_progression_compiles.rs",
            ),
        ],
    )
}
