use std::any::type_name;

use worth_proof::{FamilyResolvedReference, ForkOutputs2, JoinInputs2, Pair};

use super::super::proof_shapes::{FailureDigest, ProofShapeDigest, TransitionDigest};
use super::representatives::{
    JoinedReadyRecipe, LeftReadyRecipe, RepresentativeFamilyAction,
    RepresentativeLoweredFamilyProgram, RepresentativeMember, RepresentativeSymbol,
    RightReadyRecipe,
};

pub fn transition_digest() -> TransitionDigest {
    TransitionDigest::new(
        "static_fork_join_and_composition_family",
        vec![
            type_name::<
                fn(
                    worth_proof::TransitionOutcome<LeftReadyRecipe, &'static str>,
                    fn() -> worth_proof::TransitionOutcome<RightReadyRecipe, &'static str>,
                    fn(
                        JoinInputs2<LeftReadyRecipe, RightReadyRecipe>,
                    )
                        -> worth_proof::TransitionOutcome<JoinedReadyRecipe, &'static str>,
                )
                    -> worth_proof::TransitionOutcome<JoinedReadyRecipe, &'static str>,
            >(),
            type_name::<
                fn(
                    worth_proof::TransitionOutcome<LeftReadyRecipe, &'static str>,
                    fn() -> worth_proof::TransitionOutcome<RightReadyRecipe, &'static str>,
                    fn(
                        JoinInputs2<LeftReadyRecipe, RightReadyRecipe>,
                    )
                        -> worth_proof::SuccessfulTransitionOutcome<JoinedReadyRecipe>,
                )
                    -> worth_proof::TransitionOutcome<JoinedReadyRecipe, &'static str>,
            >(),
            "compose_join_transition_outcome",
            "compose_join_success_transition",
            "compose_join_ready_recipe_pair",
            "join_ready_recipe_pair",
            "non_success_short_circuit::left_denial",
            "non_success_short_circuit::right_denial",
            "family_lifecycle::create",
            "family_lifecycle::rewrite",
            "family_lifecycle::supersede",
            "family_lifecycle::retire",
            "deterministic_family_lowering::create_retire_converges",
            "deterministic_family_lowering::rewrite_supersede_converges",
            "family_identity_boundary::symbolic_and_authoritative_are_distinct",
        ],
    )
}

pub fn composition_digest() -> ProofShapeDigest {
    ProofShapeDigest::new(
        "static_fork_join_and_composition_family",
        vec![
            type_name::<ForkOutputs2<u64, u16>>(),
            type_name::<JoinInputs2<u64, u16>>(),
            type_name::<LeftReadyRecipe>(),
            type_name::<RightReadyRecipe>(),
            type_name::<JoinedReadyRecipe>(),
            type_name::<RepresentativeSymbol>(),
            type_name::<RepresentativeMember>(),
            type_name::<FamilyResolvedReference<u8, u16>>(),
            type_name::<RepresentativeFamilyAction>(),
            type_name::<RepresentativeLoweredFamilyProgram>(),
            type_name::<
                fn(RepresentativeSymbol, RepresentativeMember) -> FamilyResolvedReference<u8, u16>,
            >(),
            type_name::<fn(JoinInputs2<LeftReadyRecipe, RightReadyRecipe>) -> JoinedReadyRecipe>(),
            type_name::<
                fn(
                    Pair<RepresentativeFamilyAction>,
                    fn(&RepresentativeFamilyAction) -> (u8, Option<u8>, Option<u16>),
                ) -> RepresentativeLoweredFamilyProgram,
            >(),
            "resolve_family_symbol",
            "lower_deterministic_family_pair",
        ],
    )
}

pub fn proof_shape_digest() -> ProofShapeDigest {
    composition_digest()
}

pub fn failure_digest() -> FailureDigest {
    FailureDigest::new(
        "static_fork_join_and_composition_family",
        vec![
            "wrong_join_shape::tests/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs",
            "wrong_fork_shape::tests/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs",
            "lowered_ready_join_boundary::tests/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs",
            "family_identity_boundary::tests/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs",
            "short_circuit::left_denial_skips_right_lane",
            "short_circuit::right_denial_skips_next_step",
            "equivalence_lane::create_retire_family_lowering_converges",
            "equivalence_lane::rewrite_supersede_family_lowering_converges",
            "identity_divergence::symbolic_family_reference_is_not_authoritative_identity",
        ],
    )
}
