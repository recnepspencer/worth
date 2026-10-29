use worth_proof::prelude::*;

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn admitted_flow(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    admission_authority: AuthorityWitness<AdmissionAuthority>,
) {
    let admitted = recipe("payload")
        .resolve_with(resolution_authority, 7_u8)
        .lower_with(lowering_capability)
        .admit_with(admission_authority);

    let _ = admitted.payload();
}

fn execution_flow(
    resolution_authority: AuthorityWitness<ResolutionAuthority>,
    lowering_capability: CapabilityWitness<LoweringCapability>,
    readiness_authority: AuthorityWitness<ReadinessAuthority>,
) {
    let executed = recipe("payload")
        .resolve_with(resolution_authority, 11_u8)
        .lower_with(lowering_capability)
        .ready_with(readiness_authority, "runtime admission")
        .execute();

    let _ = executed.strong_basis().value();
}

fn main() {
    let _ = admitted_flow;
    let _ = execution_flow;
}
