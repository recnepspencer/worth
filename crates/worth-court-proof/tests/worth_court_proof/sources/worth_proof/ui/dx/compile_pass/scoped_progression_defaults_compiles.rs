use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct AlternateResolutionAuthority;
impl AuthorityMarker for AlternateResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AlternateLoweringCapability;
impl CapabilityMarker for AlternateLoweringCapability {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

struct AlternateReadinessAuthority;
impl AuthorityMarker for AlternateReadinessAuthority {}

fn scoped_defaults_progression(
    default_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    default_lowering_capability: CapabilityWitness<LoweringCapability>,
    default_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    second_resolution_authority: AuthorityWitness<ResolutionAuthority>,
    second_lowering_capability: CapabilityWitness<LoweringCapability>,
    second_readiness_authority: AuthorityWitness<ReadinessAuthority>,
    alternate_resolution_authority: AuthorityWitness<AlternateResolutionAuthority>,
    alternate_lowering_capability: CapabilityWitness<AlternateLoweringCapability>,
    alternate_readiness_authority: AuthorityWitness<AlternateReadinessAuthority>,
) {
    let executed = proof_flow()
        .resolution_authority(default_resolution_authority)
        .lowering_capability(default_lowering_capability)
        .readiness_authority(default_readiness_authority)
        .recipe("payload")
        .resolve(7_u8)
        .lower()
        .ready("runtime admission")
        .execute();

    let overridden = proof_flow()
        .resolution_authority(second_resolution_authority)
        .lowering_capability(second_lowering_capability)
        .readiness_authority(second_readiness_authority)
        .recipe("payload")
        .resolve_with(alternate_resolution_authority, 11_u16)
        .lower_with(alternate_lowering_capability)
        .ready_with(alternate_readiness_authority, "runtime admission")
        .execute();

    let _ = executed.payload();
    let _ = overridden.payload();
}

fn main() {
    let _ = scoped_defaults_progression;
}
