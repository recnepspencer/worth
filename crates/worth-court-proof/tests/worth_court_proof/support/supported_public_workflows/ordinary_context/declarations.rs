// Caller-level contracts for the bounded workflows worth-proof promises to support.

pub(crate) use worth_proof::prelude::*;
pub(crate) use {worth_proof::with_brand, worth_proof::DisjointPair};

pub(crate) struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

pub(crate) struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

pub(crate) struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

pub(crate) struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

pub(crate) struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

pub(crate) fn resolution_authority() -> AuthorityWitness<ResolutionAuthority> {
    AuthorityWitness::from_authority_marker(ResolutionAuthority)
}

pub(crate) fn lowering_capability() -> CapabilityWitness<LoweringCapability> {
    CapabilityWitness::from_capability_marker(LoweringCapability)
}

pub(crate) fn admission_authority() -> AuthorityWitness<AdmissionAuthority> {
    AuthorityWitness::from_authority_marker(AdmissionAuthority)
}

pub(crate) fn readmission_authority() -> AuthorityWitness<ReadmissionAuthority> {
    AuthorityWitness::from_authority_marker(ReadmissionAuthority)
}

pub(crate) fn readiness_authority() -> AuthorityWitness<ReadinessAuthority> {
    AuthorityWitness::from_authority_marker(ReadinessAuthority)
}

pub(crate) fn checked_disjoint_pair_construction() {
    let pair = worth_proof::DisjointPair::try_from_disjoint("left", "right")
        .expect("unequal values are admitted as disjoint");
    assert_eq!((pair.left(), pair.right()), (&"left", &"right"));
    assert!(worth_proof::DisjointPair::try_from_disjoint("same", "same").is_err());
}

pub(crate) fn scoped_brand_usage() {
    let value = with_brand(|brand| brand.bind("scoped").into_value());
    assert_eq!(value, "scoped");
}

pub(crate) fn proof_and_capability_progression() {
    let admitted = recipe("payload")
        .resolve_with(resolution_authority(), 7_u8)
        .lower_with(lowering_capability())
        .admit_with(admission_authority());
    assert_eq!(admitted.payload(), &"payload");
}

pub(crate) fn recipe_resolution_through_execution_readiness() {
    let outcome = recipe("payload")
        .try_resolve_ready(7_u8, resolution_authority())
        .try_lower_ready(lowering_capability())
        .try_ready_now("runtime admission", readiness_authority())
        .try_execute();
    assert_eq!(outcome.kind(), ProofOutcomeKind::Success);
}

pub(crate) fn trust_boundary_bridging() {
    let executed = recipe("payload")
        .resolve_with(resolution_authority(), 11_u8)
        .lower_with(lowering_capability())
        .bridge_trust_boundary()
        .readmit_with(readmission_authority(), 19_u16)
        .ready_with(readiness_authority(), "runtime admission")
        .execute();
    assert_eq!(executed.strong_basis().value(), &19_u16);
}

pub(crate) fn primary_transition_workflow() {
    let executed = proof_flow()
        .resolution_authority(resolution_authority())
        .lowering_capability(lowering_capability())
        .readiness_authority(readiness_authority())
        .recipe("payload")
        .resolve(23_u8)
        .lower()
        .ready("runtime admission")
        .execute();
    assert_eq!(executed.payload(), &"payload");
}
