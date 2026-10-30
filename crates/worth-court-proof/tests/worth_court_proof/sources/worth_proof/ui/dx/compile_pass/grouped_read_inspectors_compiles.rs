use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn grouped_reads(
    resolved_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    ready_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    ready_lowering_capability: CapabilityWitness<LoweringCapability>,
    ready_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    left_join_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    left_join_lowering_capability: CapabilityWitness<LoweringCapability>,
    left_join_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    right_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    right_lowering_capability: CapabilityWitness<LoweringCapability>,
    right_readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let unresolved = recipe("payload");
    let resolved = recipe("payload").resolve_with(resolved_resolution_authority, 7_u8);
    let ready = recipe("payload")
        .resolve_with(ready_resolution_authority, 7_u8)
        .lower_with(ready_lowering_capability)
        .ready_with(ready_readiness_authority, "runtime admission");
    let joined = join_ready(
        recipe("left")
            .resolve_with(left_join_resolution_authority, 9_u8)
            .lower_with(left_join_lowering_capability)
            .ready_with(left_join_readiness_authority, "runtime admission"),
        recipe("right")
            .resolve_with(right_resolution_authority, 11_u16)
            .lower_with(right_lowering_capability)
            .ready_with(right_readiness_authority, "runtime admission"),
    );
    let lowered_family = family_pair(
        create::<u8, u16, _>(sym(2_u8), "create"),
        retire::<u8, u16, &'static str>(member(11_u16)),
    )
    .lower_by(family_action_key);

    let _ = unresolved.stage();
    let _ = resolved.basis_posture();
    let _ = ready.has_strong_basis();
    let summary = joined.summary();
    let _ = summary.left_payload();
    let _ = lowered_family.action_kinds();
}

fn family_action_key(
    action: &worth_proof::FamilyLifecycleAction<u8, u16, &'static str>,
) -> (u8, Option<u8>, Option<u16>) {
    match action {
        worth_proof::FamilyLifecycleAction::Retire { target } => (0, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Rewrite { target, .. } => {
            (1, None, Some(*target.value()))
        }
        worth_proof::FamilyLifecycleAction::Supersede { target, .. } => {
            (2, None, Some(*target.value()))
        }
        worth_proof::FamilyLifecycleAction::Create { symbol, .. } => {
            (3, Some(*symbol.value()), None)
        }
    }
}

fn main() {
    let _ = grouped_reads;
}
