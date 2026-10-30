use worth_proof::{AuthorityMarker, AuthorityWitness, CapabilityMarker, CapabilityWitness};

struct DeploymentAuthority;
impl AuthorityMarker for DeploymentAuthority {}

struct CanonicalizationCapability;
impl CapabilityMarker for CanonicalizationCapability {}

fn main() {
    let _authority = AuthorityWitness::<DeploymentAuthority>::mint();
    let _capability = CapabilityWitness::<CanonicalizationCapability>::mint();
}
// sealed-minting-case
