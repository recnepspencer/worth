use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn cannot_skip_from_unresolved_to_lowered(
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let _ = recipe("payload").lower_with(lowering_capability);
}

fn cannot_skip_from_unresolved_to_ready(
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let _ = recipe("payload").ready_with(readiness_authority, "runtime admission");
}

fn cannot_execute_without_ready(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let _ = recipe("payload")
        .resolve_with(resolution_authority, 7_u8)
        .lower_with(lowering_capability)
        .execute();
}

fn main() {}
