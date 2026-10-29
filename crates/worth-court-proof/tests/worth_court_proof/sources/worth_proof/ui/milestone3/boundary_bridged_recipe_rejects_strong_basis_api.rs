use worth_proof::{AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness, Recipe};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn invalid_pre_readmission_strong_basis_access(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let admitted = Recipe::new("payload")
        .resolve_with_authority(7_u8, resolution_authority)
        .lower_with_capability(lowering_capability)
        .admit_with_authority(admission_authority);
    let bridged = admitted.bridge_trust_boundary();

    let _basis = bridged.strong_basis();
}

fn main() {}
