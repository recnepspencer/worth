// Caller-level contracts for the bounded workflows worth-proof promises to support.

use worth_proof::prelude::*;
use worth_proof::{with_brand, DisjointPair};

struct ResolutionAuthority;
impl AuthorityMarker for ResolutionAuthority {}

struct LoweringCapability;
impl CapabilityMarker for LoweringCapability {}

struct AdmissionAuthority;
impl AuthorityMarker for AdmissionAuthority {}

struct ReadmissionAuthority;
impl AuthorityMarker for ReadmissionAuthority {}

struct ReadinessAuthority;
impl AuthorityMarker for ReadinessAuthority {}

fn resolution_authority() -> AuthorityWitness<ResolutionAuthority> {
    AuthorityWitness::from_authority_marker(ResolutionAuthority)
}

fn lowering_capability() -> CapabilityWitness<LoweringCapability> {
    CapabilityWitness::from_capability_marker(LoweringCapability)
}

fn admission_authority() -> AuthorityWitness<AdmissionAuthority> {
    AuthorityWitness::from_authority_marker(AdmissionAuthority)
}

fn readmission_authority() -> AuthorityWitness<ReadmissionAuthority> {
    AuthorityWitness::from_authority_marker(ReadmissionAuthority)
}

fn readiness_authority() -> AuthorityWitness<ReadinessAuthority> {
    AuthorityWitness::from_authority_marker(ReadinessAuthority)
}

#[test]
fn supported_public_workflow_catalog_executes_at_caller_altitude() {
    checked_disjoint_pair_construction();
    scoped_brand_usage();
    proof_and_capability_progression();
    recipe_resolution_through_execution_readiness();
    trust_boundary_bridging();
    primary_transition_workflow();
}

fn checked_disjoint_pair_construction() {
    let pair = DisjointPair::try_from_disjoint("left", "right")
        .expect("unequal values are admitted as disjoint");
    assert_eq!((pair.left(), pair.right()), (&"left", &"right"));
    assert!(DisjointPair::try_from_disjoint("same", "same").is_err());
}

fn scoped_brand_usage() {
    let value = with_brand(|brand| brand.bind("scoped").into_value());
    assert_eq!(value, "scoped");
}

fn proof_and_capability_progression() {
    let admitted = recipe("payload")
        .resolve_with(resolution_authority(), 7_u8)
        .lower_with(lowering_capability())
        .admit_with(admission_authority());
    assert_eq!(admitted.payload(), &"payload");
}

fn recipe_resolution_through_execution_readiness() {
    let outcome = recipe("payload")
        .try_resolve_ready(7_u8, resolution_authority())
        .try_lower_ready(lowering_capability())
        .try_ready_now("runtime admission", readiness_authority())
        .try_execute();
    assert_eq!(outcome.kind(), ProofOutcomeKind::Success);
}

fn trust_boundary_bridging() {
    let executed = recipe("payload")
        .resolve_with(resolution_authority(), 11_u8)
        .lower_with(lowering_capability())
        .bridge_trust_boundary()
        .readmit_with(readmission_authority(), 19_u16)
        .ready_with(readiness_authority(), "runtime admission")
        .execute();
    assert_eq!(executed.strong_basis().value(), &19_u16);
}

fn primary_transition_workflow() {
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
