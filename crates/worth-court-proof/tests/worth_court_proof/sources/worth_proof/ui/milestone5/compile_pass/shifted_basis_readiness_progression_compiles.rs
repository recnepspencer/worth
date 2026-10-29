use worth_proof::{
    AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness,
    CheckedReadmitLoweredForExecutionReadyTransition, ContextualTransition,
    LowerRecipeTransition, LoweredReadmissionContext, LoweredReadmissionReadiness, Recipe,
    RecipeResolutionContext, ResolveRecipeTransition, Transition, Unresolved,
    checked_readmit_ready_and_execute_recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

type ShiftedReadmissionReadiness = LoweredReadmissionReadiness<
    &'static str,
    u8,
    u16,
    ReadmissionAuthority,
    &'static str,
    ReadinessAuthority,
    &'static str,
    &'static str,
    &'static str,
>;

fn shifted_basis_runtime_readmission_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readmission_authority: AuthorityWitness<ReadmissionAuthority>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(12_u8, resolution_authority),
    );
    let lowered = LowerRecipeTransition::new(lowering_capability)
        .transition(resolved.into_value())
        .into_value();

    let ready = CheckedReadmitLoweredForExecutionReadyTransition.transition(
        lowered.bridge_trust_boundary(),
        ShiftedReadmissionReadiness::ready(LoweredReadmissionContext::new(
            19_u16,
            readmission_authority,
            "runtime admission",
            readiness_authority,
        )),
    );

    let _ = ready;
}

fn shifted_basis_runtime_readmission_execute_progression_compiles(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readmission_authority: AuthorityWitness<ReadmissionAuthority>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let unresolved = Recipe::<Unresolved, _>::new("payload");
    let resolved = ResolveRecipeTransition.transition(
        unresolved,
        RecipeResolutionContext::new(12_u8, resolution_authority),
    );
    let lowered = LowerRecipeTransition::new(lowering_capability)
        .transition(resolved.into_value())
        .into_value();

    let executed = checked_readmit_ready_and_execute_recipe(
        lowered.bridge_trust_boundary(),
        ShiftedReadmissionReadiness::ready(LoweredReadmissionContext::new(
            19_u16,
            readmission_authority,
            "runtime admission",
            readiness_authority,
        )),
    );

    let _ = executed;
}

fn main() {}
