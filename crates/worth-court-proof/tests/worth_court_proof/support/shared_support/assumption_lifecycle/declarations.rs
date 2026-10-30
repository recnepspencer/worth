pub(crate) use {
    crate::shared_support::compile_fail::bundle::CompileFailBundle,
    crate::shared_support::compile_fail::bundle::CompileFailCase,
};
pub(crate) use {
    crate::shared_support::compile_pass::bundle::CompilePassBundle,
    crate::shared_support::compile_pass::bundle::CompilePassCase,
};

pub(crate) fn compile_fail_bundle() -> CompileFailBundle {
    crate::shared_support::compile_fail::bundle::CompileFailBundle::new(
        "assumption_freshness_readmission_and_downgrade",
        vec![
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "unresolved_misuse",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/unresolved_recipe_cannot_bridge_trust_boundary.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "stale_misuse",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/stale_readable_recipe_rejects_strong_basis_api.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "rebind_misuse",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/rebind_required_recipe_cannot_lower.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "pre_readmission_misuse",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/boundary_bridged_recipe_rejects_strong_basis_api.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "basis_drift_misuse",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/shifted_basis_readmission_cannot_be_treated_as_original_basis.rs"),
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "assumption_freshness_readmission_and_downgrade",
        vec![
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "control_progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/explicit_current_validity_progression_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "same_basis_readmission_progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/explicit_same_basis_readmission_progression_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "shifted_basis_readmission_progression",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone3/explicit_readmission_progression_compiles.rs"),
            ),
        ],
    )
}
