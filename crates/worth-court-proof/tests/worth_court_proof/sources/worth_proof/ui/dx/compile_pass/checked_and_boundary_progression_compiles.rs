use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn checked_flow(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let outcome = recipe("payload")
        .try_resolve_ready(7_u8, resolution_authority)
        .try_lower_ready(lowering_capability)
        .try_ready_now("runtime admission", readiness_authority)
        .try_execute();

    let _ = outcome.kind();
}

fn trust_boundary_flow(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readmission_authority: AuthorityWitness<ReadmissionAuthority>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let executed = recipe("payload")
        .resolve_with(resolution_authority, 11_u8)
        .lower_with(lowering_capability)
        .bridge_trust_boundary()
        .readmit_with(readmission_authority, 19_u16)
        .ready_with(readiness_authority, "runtime admission")
        .execute();

    let _ = executed.strong_basis().value();
}

fn main() {
    let _ = checked_flow;
    let _ = trust_boundary_flow;
}
