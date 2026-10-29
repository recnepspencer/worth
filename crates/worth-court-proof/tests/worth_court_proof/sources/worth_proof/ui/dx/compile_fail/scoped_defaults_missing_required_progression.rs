use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn missing_resolution_default(
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let _ = proof_flow()
        .lowering_capability(lowering_capability)
        .readiness_authority(readiness_authority)
        .recipe("payload")
        .resolve(7_u8);
}

fn missing_lowering_default(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let _ = proof_flow()
        .resolution_authority(resolution_authority)
        .readiness_authority(readiness_authority)
        .recipe("payload")
        .resolve(7_u8)
        .lower();
}

fn missing_readiness_default(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let _ = proof_flow()
        .resolution_authority(resolution_authority)
        .lowering_capability(lowering_capability)
        .recipe("payload")
        .resolve(7_u8)
        .lower()
        .ready("runtime admission");
}

fn main() {}
