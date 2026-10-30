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
        "static_fork_join_and_composition_family",
        vec![
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "wrong_join_shape",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "wrong_fork_shape",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "lowered_ready_join_boundary",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs"),
            ),
            crate::shared_support::compile_fail::bundle::CompileFailCase::new(
                "family_identity_boundary",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs"),
            ),
        ],
    )
}

pub(crate) fn compile_pass_bundle() -> CompilePassBundle {
    crate::shared_support::compile_pass::bundle::CompilePassBundle::new(
        "static_fork_join_and_composition_family",
        vec![
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "explicit_fixed_arity_fork_join",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/explicit_fixed_arity_fork_join_progression_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "checked_multi_input_ordering_and_ready_join",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/checked_multi_input_ordering_and_ready_join_compiles.rs"),
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "explicit_family_symbol_resolution_and_lowering",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/explicit_family_symbol_resolution_and_lowering_compiles.rs"),
            ),
        ],
    )
}
