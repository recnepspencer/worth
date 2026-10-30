use worth_proof::{
    AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness, Recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

fn invalid_stale_strong_basis_access(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
) {
    let resolved = Recipe::new("payload").resolve_with_authority(7_u8, resolution_authority);
    let lowered = resolved.lower_with_capability(lowering_capability);
    let stale = lowered.downgrade_to_stale_readable();

    let _basis = stale.strong_basis();
}

fn main() {}
