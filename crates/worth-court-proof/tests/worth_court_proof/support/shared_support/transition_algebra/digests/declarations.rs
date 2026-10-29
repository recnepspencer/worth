pub(crate) use {std::any::type_name};

pub(crate) use {worth_proof::AdmitRecipeTransition, worth_proof::Admitted, worth_proof::AssumptionBasis, worth_proof::CheckedAdmitRecipeTransition, worth_proof::CheckedLowerRecipeTransition, worth_proof::CheckedResolveRecipeTransition, worth_proof::CurrentValidity, worth_proof::DeferredTransitionOutcome, worth_proof::DenialTransitionOutcome, worth_proof::FreshnessScopedBasis, worth_proof::FreshnessTransitionOutcome, worth_proof::LowerRecipeTransition, worth_proof::Lowered, worth_proof::PreConstructionGate, worth_proof::Recipe, worth_proof::RecipeAdmissionReadiness, worth_proof::RecipeLoweringReadiness, worth_proof::RecipeResolutionContext, worth_proof::RecipeResolutionGate, worth_proof::ResolveRecipeTransition, worth_proof::Resolved, worth_proof::SuccessfulTransitionOutcome, worth_proof::TransitionOutcome, worth_proof::TransitionReadiness, worth_proof::Unresolved};

pub(crate) use {crate::shared_support::proof_shapes::digest::FailureDigest, crate::shared_support::proof_shapes::digest::TransitionDigest};
pub(crate) use {crate::shared_support::transition_algebra::representatives::RepresentativeAuthority};

pub(crate) fn transition_digest() -> TransitionDigest {
    crate::shared_support::proof_shapes::digest::TransitionDigest::new(
        "transition_outcome_algebra",
        vec![
            type_name::<ResolveRecipeTransition>(),
            type_name::<CheckedResolveRecipeTransition>(),
            type_name::<
                CheckedLowerRecipeTransition<
                    crate::shared_support::sealed_minting::RepresentativeCapability,
                >,
            >(),
            type_name::<CheckedAdmitRecipeTransition<RepresentativeAuthority>>(),
            type_name::<
                LowerRecipeTransition<crate::shared_support::sealed_minting::RepresentativeCapability>,
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
                    crate::shared_support::sealed_minting::RepresentativeCapability,
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
                    &LowerRecipeTransition<crate::shared_support::sealed_minting::RepresentativeCapability>,
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
                        crate::shared_support::sealed_minting::RepresentativeCapability,
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

pub(crate) fn failure_digest() -> FailureDigest {
    crate::shared_support::proof_shapes::digest::FailureDigest::new(
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