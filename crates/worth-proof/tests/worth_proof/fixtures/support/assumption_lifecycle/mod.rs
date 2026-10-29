use super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "assumption_freshness_readmission_and_downgrade",
        vec![
            CompileFailCase::new(
                "unresolved_misuse",
                "tests/ui/milestone3/unresolved_recipe_cannot_bridge_trust_boundary.rs",
            ),
            CompileFailCase::new(
                "stale_misuse",
                "tests/ui/milestone3/stale_readable_recipe_rejects_strong_basis_api.rs",
            ),
            CompileFailCase::new(
                "rebind_misuse",
                "tests/ui/milestone3/rebind_required_recipe_cannot_lower.rs",
            ),
            CompileFailCase::new(
                "pre_readmission_misuse",
                "tests/ui/milestone3/boundary_bridged_recipe_rejects_strong_basis_api.rs",
            ),
            CompileFailCase::new(
                "basis_drift_misuse",
                "tests/ui/milestone3/shifted_basis_readmission_cannot_be_treated_as_original_basis.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "assumption_freshness_readmission_and_downgrade",
        vec![
            CompilePassCase::new(
                "control_progression",
                "tests/ui/milestone3/explicit_current_validity_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "same_basis_readmission_progression",
                "tests/ui/milestone3/explicit_same_basis_readmission_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "shifted_basis_readmission_progression",
                "tests/ui/milestone3/explicit_readmission_progression_compiles.rs",
            ),
        ],
    )
}
