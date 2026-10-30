use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn scoped_defaults_are_consumed_once(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let flow = proof_flow()
        .resolution_authority(resolution_authority)
        .lowering_capability(lowering_capability)
        .readiness_authority(readiness_authority);

    let _first = flow
        .recipe("first")
        .resolve(7_u8)
        .lower()
        .ready("runtime admission")
        .execute();

    let _second = flow.recipe("second");
}

fn main() {}
