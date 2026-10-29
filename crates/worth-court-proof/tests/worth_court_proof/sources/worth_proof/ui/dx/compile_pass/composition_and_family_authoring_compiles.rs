use worth_proof::prelude::*;

fn family_action_key(
    action: &worth_proof::FamilyLifecycleAction<u8, u16, &'static str>,
) -> (u8, Option<u8>, Option<u16>) {
    match action {
        worth_proof::FamilyLifecycleAction::Retire { target } => (0, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Rewrite { target, .. } => (1, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Supersede { target, .. } => (2, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Create { symbol, .. } => (3, Some(*symbol.value()), None),
    }
}

fn family_flow() {
    let lowered = family_pair(
        create::<u8, u16, _>(sym(2_u8), "create"),
        supersede::<u8, u16, _>(member(11_u16), sym(3_u8), "replace"),
    )
    .lower_by(family_action_key);

    let _ = lowered.actions().left();
}

fn ready_join_flow(
    left_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    left_lowering_capability: CapabilityWitness<LoweringCapability>,
    left_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    right_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    right_lowering_capability: CapabilityWitness<LoweringCapability>,
    right_readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let left = recipe("left")
        .resolve_with(left_resolution_authority, 7_u8)
        .lower_with(left_lowering_capability)
        .ready_with(left_readiness_authority, "runtime admission");
    let right = recipe("right")
        .resolve_with(right_resolution_authority, 9_u16)
        .lower_with(right_lowering_capability)
        .ready_with(right_readiness_authority, "runtime admission");

    let joined = join_ready(left, right);
    let _ = joined.payload().left();
}

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn compose_ready_flow(
    left_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    left_lowering_capability: CapabilityWitness<LoweringCapability>,
    left_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    right_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    right_lowering_capability: CapabilityWitness<LoweringCapability>,
    right_readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let left = recipe("left")
        .resolve_with(left_resolution_authority, 7_u8)
        .lower_with(left_lowering_capability)
        .ready_with(left_readiness_authority, "runtime admission");
    let right = recipe("right")
        .resolve_with(right_resolution_authority, 9_u16)
        .lower_with(right_lowering_capability)
        .ready_with(right_readiness_authority, "runtime admission");

    let joined = compose_ready(
        worth_proof::TransitionOutcome::<_, &'static str>::success(left),
        || worth_proof::TransitionOutcome::success(right),
    );

    let _ = joined;
}

fn main() {
    let _ = family_flow;
    let _ = ready_join_flow;
    let _ = compose_ready_flow;
}
