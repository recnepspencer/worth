use worth_proof::{
    AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CheckedAdmitExecutionReadyRecipeTransition, ContextualTransition,
    ExecutionReadinessContext, ExecutionReadyAdmissionReadiness, LowerRecipeTransition, Recipe,
    RecipeResolutionContext, ResolveRecipeTransition, Transition, Unresolved,
    checked_admit_ready_and_execute_recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

type CurrentLoweredRecipe = worth_proof::Recipe<
    worth_proof::Lowered,
    &'static str,
    worth_proof::FreshnessScopedBasis<
        worth_proof::CurrentValidity,
        worth_proof::AssumptionBasis<u8>,
    >,
>;

type CurrentResolvedRecipe = worth_proof::Recipe<
    worth_proof::Resolved,
    &'static str,
    worth_proof::FreshnessScopedBasis<
        worth_proof::CurrentValidity,
        worth_proof::AssumptionBasis<u8>,
    >,
>;

type StaleLoweredRecipe = worth_proof::Recipe<
    worth_proof::Lowered,
    &'static str,
    worth_proof::StaleReadableBasis<u8>,
>;

type RebindResolvedRecipe = worth_proof::Recipe<
    worth_proof::Resolved,
    &'static str,
    worth_proof::RebindRequiredBasis<u8>,
>;

type CheckedReadiness = ExecutionReadyAdmissionReadiness<
    &'static str,
    u8,
    &'static str,
    ReadinessAuthority,
    &'static str,
    &'static str,
    &'static str,
>;

fn lower_recipe(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) -> CurrentLoweredRecipe {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(12_u8, resolution_authority),
    );

    LowerRecipeTransition::new(lowering_capability)
        .transition(resolved.into_value())
        .into_value()
}

fn resolve_recipe(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
) -> CurrentResolvedRecipe {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    ResolveRecipeTransition
        .transition(
            unresolved,
            RecipeResolutionContext::new(12_u8, resolution_authority),
        )
        .into_value()
}

fn checked_ready_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let lowered = lower_recipe(resolution_authority, lowering_capability);
    let ready = CheckedAdmitExecutionReadyRecipeTransition.transition(
        lowered,
        CheckedReadiness::ready(ExecutionReadinessContext::new(
            "runtime admission",
            readiness_authority,
        )),
    );

    let _ = ready;
}

fn checked_ready_execute_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let lowered = lower_recipe(resolution_authority, lowering_capability);
    let executed = checked_admit_ready_and_execute_recipe(
        lowered,
        CheckedReadiness::ready(ExecutionReadinessContext::new(
            "runtime admission",
            readiness_authority,
        )),
    );

    let _ = executed;
}

fn checked_stale_progression_compiles(
    resolution_authority_for_lowered: AuthorityWitness<ResolutionAuthority>,
    lowering_capability_for_lowered: CapabilityWitness<LoweringCapability>,
    resolution_authority_for_stale: AuthorityWitness<ResolutionAuthority>,
    lowering_capability_for_stale: CapabilityWitness<LoweringCapability>,
) {
    let lowered = lower_recipe(
        resolution_authority_for_lowered,
        lowering_capability_for_lowered,
    );
    let stale_lowered: StaleLoweredRecipe = lower_recipe(
        resolution_authority_for_stale,
        lowering_capability_for_stale,
    )
    .downgrade_to_stale_readable();

    let stale = CheckedAdmitExecutionReadyRecipeTransition.transition(
        lowered,
        CheckedReadiness::stale(stale_lowered),
    );

    let _ = stale;
}

fn checked_rebind_progression_compiles(
    resolution_authority_for_lowered: AuthorityWitness<ResolutionAuthority>,
    lowering_capability_for_lowered: CapabilityWitness<LoweringCapability>,
    resolution_authority_for_rebind: AuthorityWitness<ResolutionAuthority>,
) {
    let lowered = lower_recipe(
        resolution_authority_for_lowered,
        lowering_capability_for_lowered,
    );
    let rebind_resolved: RebindResolvedRecipe =
        resolve_recipe(resolution_authority_for_rebind).downgrade_to_rebind_required();

    let rebind = CheckedAdmitExecutionReadyRecipeTransition.transition(
        lowered,
        CheckedReadiness::rebind_required(rebind_resolved),
    );

    let _ = rebind;
}

fn main() {}
