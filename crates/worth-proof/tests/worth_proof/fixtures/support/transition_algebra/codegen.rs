use std::mem::{align_of, needs_drop};

use worth_proof::{
    AssumptionBasis, CheckedAdmitRecipeTransition, CheckedLowerRecipeTransition, CurrentValidity,
    DeferredTransitionOutcome, DenialTransitionOutcome, FreshnessScopedBasis,
    FreshnessTransitionOutcome, Recipe, RecipeResolutionContext, RecipeResolutionGate, Resolved,
    SuccessfulTransitionOutcome, TransitionOutcome, TransitionReadiness,
};

use super::super::type_shapes::{CodegenHonestyReport, CodegenShapeCheck};
use super::representatives::RepresentativeAuthority;

pub fn codegen_honesty_report() -> CodegenHonestyReport {
    CodegenHonestyReport::size_layout_and_drop_certified(
        "transition_outcome_algebra",
        vec![
            CodegenShapeCheck::new(
                "resolve_transition",
                align_of::<worth_proof::ResolveRecipeTransition>(),
                align_of::<()>(),
                needs_drop::<worth_proof::ResolveRecipeTransition>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "resolution_context",
                align_of::<RecipeResolutionContext<u8, RepresentativeAuthority>>(),
                align_of::<u8>(),
                needs_drop::<RecipeResolutionContext<u8, RepresentativeAuthority>>(),
                needs_drop::<u8>(),
            ),
            CodegenShapeCheck::new(
                "pre_construction_gate",
                align_of::<RecipeResolutionGate<
                    u8,
                    RepresentativeAuthority,
                    &'static str,
                    &'static str,
                >>(),
                align_of::<&'static str>(),
                needs_drop::<RecipeResolutionGate<
                    u8,
                    RepresentativeAuthority,
                    &'static str,
                    &'static str,
                >>(),
                needs_drop::<RecipeResolutionContext<u8, RepresentativeAuthority>>(),
            ),
            CodegenShapeCheck::new(
                "transition_readiness",
                align_of::<TransitionReadiness<
                    u64,
                    &'static str,
                    &'static str,
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        worth_proof::Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >>(),
                align_of::<
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                >(),
                needs_drop::<TransitionReadiness<
                    u64,
                    &'static str,
                    &'static str,
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        worth_proof::Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >>(),
                needs_drop::<
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                >(),
            ),
            CodegenShapeCheck::new(
                "checked_lower_transition",
                align_of::<CheckedLowerRecipeTransition<super::super::sealed_minting::RepresentativeCapability>>(),
                align_of::<()>(),
                needs_drop::<
                    CheckedLowerRecipeTransition<super::super::sealed_minting::RepresentativeCapability>,
                >(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "checked_admit_transition",
                align_of::<CheckedAdmitRecipeTransition<RepresentativeAuthority>>(),
                align_of::<()>(),
                needs_drop::<CheckedAdmitRecipeTransition<RepresentativeAuthority>>(),
                needs_drop::<()>(),
            ),
            CodegenShapeCheck::new(
                "success_outcome",
                align_of::<SuccessfulTransitionOutcome<
                    Recipe<Resolved, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >>(),
                align_of::<
                    Recipe<Resolved, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
                needs_drop::<SuccessfulTransitionOutcome<
                    Recipe<Resolved, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >>(),
                needs_drop::<
                    Recipe<Resolved, u64, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>>,
                >(),
            ),
            CodegenShapeCheck::new(
                "freshness_outcome",
                align_of::<FreshnessTransitionOutcome<
                    u64,
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        worth_proof::Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >>(),
                align_of::<
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                >(),
                needs_drop::<FreshnessTransitionOutcome<
                    u64,
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                    Recipe<
                        worth_proof::Resolved,
                        u64,
                        FreshnessScopedBasis<worth_proof::RebindRequired, AssumptionBasis<u8>>,
                    >,
                    &'static str,
                >>(),
                needs_drop::<
                    Recipe<
                        worth_proof::Lowered,
                        u64,
                        FreshnessScopedBasis<worth_proof::StaleReadable, AssumptionBasis<u8>>,
                    >,
                >(),
            ),
            CodegenShapeCheck::new(
                "denial_outcome",
                align_of::<DenialTransitionOutcome<u64, &'static str>>(),
                align_of::<u64>(),
                needs_drop::<DenialTransitionOutcome<u64, &'static str>>(),
                needs_drop::<u64>(),
            ),
            CodegenShapeCheck::new(
                "deferred_outcome",
                align_of::<DeferredTransitionOutcome<u64, &'static str, &'static str>>(),
                align_of::<u64>(),
                needs_drop::<DeferredTransitionOutcome<u64, &'static str, &'static str>>(),
                needs_drop::<u64>(),
            ),
            CodegenShapeCheck::new(
                "generic_outcome",
                align_of::<
                    TransitionOutcome<
                        u64,
                        &'static str,
                        &'static str,
                        &'static str,
                        &'static str,
                        &'static str,
                    >,
                >(),
                align_of::<u64>(),
                needs_drop::<
                    TransitionOutcome<
                        u64,
                        &'static str,
                        &'static str,
                        &'static str,
                        &'static str,
                        &'static str,
                    >,
                >(),
                needs_drop::<u64>(),
            ),
        ],
        "Milestone 4 currently certifies representative size/layout/drop honesty for the transition vocabulary, checked readiness carriers, and static transition surfaces; it does not yet ship exhaustive domain-integration codegen baselines.",
    )
}
