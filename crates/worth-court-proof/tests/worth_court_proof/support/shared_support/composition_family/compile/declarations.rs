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
        "static_fork_join_and_composition_family",
        vec![
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "wrong_join_shape",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "wrong_fork_shape",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "lowered_ready_join_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs",
            ),
            crate::compile_fail::harness::bundle::CompileFailCase::new(
                "family_identity_boundary",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs",
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
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/explicit_fixed_arity_fork_join_progression_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "checked_multi_input_ordering_and_ready_join",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/checked_multi_input_ordering_and_ready_join_compiles.rs",
            ),
            crate::shared_support::compile_pass::bundle::CompilePassCase::new(
                "explicit_family_symbol_resolution_and_lowering",
                "tests/worth_court_proof/sources/worth_proof/ui/milestone6/compile_pass/explicit_family_symbol_resolution_and_lowering_compiles.rs",
            ),
        ],
    )
}
