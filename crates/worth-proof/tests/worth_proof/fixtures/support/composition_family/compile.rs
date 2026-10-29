use super::super::compile_fail::{CompileFailBundle, CompileFailCase};
use super::super::compile_pass::{CompilePassBundle, CompilePassCase};

pub fn compile_fail_bundle() -> CompileFailBundle {
    CompileFailBundle::new(
        "static_fork_join_and_composition_family",
        vec![
            CompileFailCase::new(
                "wrong_join_shape",
                "tests/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs",
            ),
            CompileFailCase::new(
                "wrong_fork_shape",
                "tests/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs",
            ),
            CompileFailCase::new(
                "lowered_ready_join_boundary",
                "tests/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs",
            ),
            CompileFailCase::new(
                "family_identity_boundary",
                "tests/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs",
            ),
        ],
    )
}

pub fn compile_pass_bundle() -> CompilePassBundle {
    CompilePassBundle::new(
        "static_fork_join_and_composition_family",
        vec![
            CompilePassCase::new(
                "explicit_fixed_arity_fork_join",
                "tests/ui/milestone6/compile_pass/explicit_fixed_arity_fork_join_progression_compiles.rs",
            ),
            CompilePassCase::new(
                "checked_multi_input_ordering_and_ready_join",
                "tests/ui/milestone6/compile_pass/checked_multi_input_ordering_and_ready_join_compiles.rs",
            ),
            CompilePassCase::new(
                "explicit_family_symbol_resolution_and_lowering",
                "tests/ui/milestone6/compile_pass/explicit_family_symbol_resolution_and_lowering_compiles.rs",
            ),
        ],
    )
}
