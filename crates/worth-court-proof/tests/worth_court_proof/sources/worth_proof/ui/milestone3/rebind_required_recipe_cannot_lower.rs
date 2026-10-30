use worth_proof::{
    AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness, Recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

fn invalid_rebind_progression(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let resolved = Recipe::new("payload").resolve_with_authority(7_u8, resolution_authority);
    let rebind_required = resolved.downgrade_to_rebind_required();

    let _lowered = rebind_required.lower_with_capability(lowering_capability);
}

fn main() {}
