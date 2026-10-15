use crate::shared_support::proof_shapes::{FailureDigest, ProofShapeDigest, TransitionDigest};

pub fn proof_shape_digest() -> ProofShapeDigest {
    ProofShapeDigest::new(
        "pleasant_lane_surface",
        vec![
            "entrypoint:prelude_and_helper_constructors",
            "progression:verb_first_recipe_lane",
            "checked:proof_outcome_and_try_verbs",
            "boundary:explicit_rebind_and_readmit_verbs",
            "composition:ready_join_and_family_pair_lowering",
            "reads:grouped_stage_basis_and_family_summary_inspectors",
            "scoped_defaults:explicit_witness_and_capability_carriage",
            "escape_hatch:raw_module_reexports_semantic_substrate",
        ],
    )
}

pub fn transition_digest() -> TransitionDigest {
    TransitionDigest::new(
        "pleasant_lane_representative_workflows",
        vec![
            "workflow:happy_path_recipe_progression",
            "workflow:checked_progression_and_boundary_resume",
            "workflow:fixed_arity_ready_join",
            "workflow:deterministic_family_lowering",
            "workflow:scoped_default_progression",
        ],
    )
}

pub fn failure_digest() -> FailureDigest {
    FailureDigest::new(
        "pleasant_lane_compile_time_boundaries",
        vec![
            "compile_fail:pleasant_lane_cannot_skip_progression",
            "compile_fail:missing_scoped_defaults",
            "compile_fail:consumed_scoped_defaults_cannot_be_reused",
        ],
    )
}
