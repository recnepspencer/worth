use worth_proof::{
    Admitted, AssumptionBasis, AuthorityMarker, AuthorityWitness, CapabilityMarker,
    CapabilityWitness, Lowered, Recipe, Resolved,
};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

fn invalid_stage_skips(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
    second_admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let unresolved = Recipe::new("payload");
    let resolved = unresolved.resolve_with_authority(1_u8, resolution_authority);
    let _lowered = resolved.lower_with_capability(lowering_capability);
    let _skipped = unresolved.admit_with_authority(admission_authority);
    let _also_skipped = resolved.admit_with_authority(second_admission_authority);

    let _direct_resolved =
        Recipe::<Resolved, _, _>::with_stage("payload", AssumptionBasis::new(1_u8));
    let _direct_lowered =
        Recipe::<Lowered, _, _>::with_stage("payload", AssumptionBasis::new(1_u8));
    let _direct_admitted =
        Recipe::<Admitted, _, _>::with_stage("payload", AssumptionBasis::new(1_u8));
}

fn main() {}
// sealed-minting-case
