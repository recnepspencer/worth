use std::any::type_name;

use worth_proof::{
    AdmitRecipeTransition, Admitted, AssumptionBasis, CheckedAdmitRecipeTransition,
    CheckedLowerRecipeTransition, CheckedResolveRecipeTransition, CurrentValidity,
    DeferredTransitionOutcome, DenialTransitionOutcome, FreshnessScopedBasis,
    FreshnessTransitionOutcome, LowerRecipeTransition, Lowered, PreConstructionGate, Recipe,
    RecipeAdmissionReadiness, RecipeLoweringReadiness, RecipeResolutionContext,
    RecipeResolutionGate, ResolveRecipeTransition, Resolved, SuccessfulTransitionOutcome,
    TransitionOutcome, TransitionReadiness, Unresolved,
};

use super::super::proof_shapes::{FailureDigest, TransitionDigest};
use super::representatives::RepresentativeAuthority;

pub fn transition_digest() -> TransitionDigest {
    TransitionDigest::new(
        "transition_outcome_algebra",
        vec![
            type_name::<ResolveRecipeTransition>(),
            type_name::<CheckedResolveRecipeTransition>(),
            type_name::<
                CheckedLowerRecipeTransition<
                    super::super::sealed_minting::RepresentativeCapability,
                >,
            >(),
            type_name::<CheckedAdmitRecipeTransition<RepresentativeAuthority>>(),
            type_name::<
                LowerRecipeTransition<super::super::sealed_minting::RepresentativeCapability>,
            >(),
            type_name::<AdmitRecipeTransition<RepresentativeAuthority>>(),
            type_name::<RecipeResolutionContext<u8, RepresentativeAuthority>>(),
            type_name::<
                RecipeResolutionGate<u8, RepresentativeAuthority, &'static str, &'static str>,
            >(),
            type_name::<
                RecipeLoweringReadiness<
                    u64,
                    u8,
                    super::super::sealed_minting::RepresentativeCapability,
                    &'static str,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<
                RecipeAdmissionReadiness<
                    u64,
                    u8,
                    RepresentativeAuthority,
                    &'static str,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<
                TransitionReadiness<
                    u64,
                    &'static str,
                    &'static str,
                    Recipe<
                        Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >,
            >(),
            type_name::<
                PreConstructionGate<
                    RecipeResolutionContext<u8, RepresentativeAuthority>,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<
                fn(
                    Recipe<Unresolved, u64>,
                    RecipeResolutionGate<u8, RepresentativeAuthority, &'static str, &'static str>,
                    &LowerRecipeTransition<super::super::sealed_minting::RepresentativeCapability>,
                    &AdmitRecipeTransition<RepresentativeAuthority>,
                ) -> DeferredTransitionOutcome<
                    Recipe<
                        Admitted,
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                    &'static str,
                >,
            >(),
            type_name::<
                fn(
                    Recipe<Unresolved, u64>,
                    RecipeResolutionGate<u8, RepresentativeAuthority, &'static str, &'static str>,
                    RecipeLoweringReadiness<
                        u64,
                        u8,
                        super::super::sealed_minting::RepresentativeCapability,
                        &'static str,
                        &'static str,
                        &'static str,
                    >,
                    RecipeAdmissionReadiness<
                        u64,
                        u8,
                        RepresentativeAuthority,
                        &'static str,
                        &'static str,
                        &'static str,
                    >,
                ) -> TransitionOutcome<
                    Recipe<
                        Admitted,
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                    &'static str,
                    Recipe<
                        Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >,
            >(),
            type_name::<
                SuccessfulTransitionOutcome<
                    Recipe<
                        Resolved,
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >,
            >(),
            type_name::<
                SuccessfulTransitionOutcome<
                    Recipe<
                        Lowered,
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >,
            >(),
            type_name::<
                SuccessfulTransitionOutcome<
                    Recipe<
                        Admitted,
                        u64,
                        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
                    >,
                >,
            >(),
            type_name::<DenialTransitionOutcome<u64, &'static str>>(),
            type_name::<DeferredTransitionOutcome<u64, &'static str, &'static str>>(),
            type_name::<
                FreshnessTransitionOutcome<
                    u64,
                    Recipe<
                        Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >,
            >(),
        ],
    )
}

pub fn failure_digest() -> FailureDigest {
    FailureDigest::new(
        "transition_outcome_algebra",
        vec![
            "ordering_misuse::tests/ui/milestone4/unresolved_recipe_cannot_lower_through_transition_contract.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_admit_through_transition_contract.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_enter_checked_resolution_pipeline.rs",
            "ordering_misuse::tests/ui/milestone4/lowered_recipe_cannot_enter_checked_lowering_pipeline.rs",
            "ordering_misuse::tests/ui/milestone4/resolved_recipe_cannot_enter_checked_admission_pipeline.rs",
            "category_divergence::denied",
            "category_divergence::deferred",
            "category_divergence::stale",
            "category_divergence::rebind_required",
            "category_divergence::failed",
            "equivalence_lane::direct_checked_all_ready",
        ],
    )
}
