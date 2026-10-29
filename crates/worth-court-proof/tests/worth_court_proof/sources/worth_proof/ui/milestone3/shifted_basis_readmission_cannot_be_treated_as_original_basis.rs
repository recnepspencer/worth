use worth_proof::{
    Admitted, AssumptionBasis, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    CapabilityWitness, CurrentValidity, FreshnessScopedBasis, Recipe,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

fn invalid_shifted_basis_reuse(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
    readmission_authority: AuthorityWitness<ReadmissionAuthority>,
) {
    let admitted = Recipe::new("payload")
        .resolve_with_authority(7_u8, resolution_authority)
        .lower_with_capability(lowering_capability)
        .admit_with_authority(admission_authority);

    let bridged = admitted.bridge_trust_boundary();
    let _readmitted: Recipe<
        Admitted,
        _,
        FreshnessScopedBasis<CurrentValidity, AssumptionBasis<u8>>,
    > = bridged.readmit_with_authority(11_u16, readmission_authority);
}

fn main() {}
