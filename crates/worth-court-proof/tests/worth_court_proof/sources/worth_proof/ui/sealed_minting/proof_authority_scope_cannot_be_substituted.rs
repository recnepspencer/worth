use worth_proof::{AuthorityMarker, AuthorityProves, AuthorityWitness, CanonicalOrder, Proof};

struct SourceAuthority;
impl AuthorityMarker for SourceAuthority {}
impl AuthorityProves<CanonicalOrder> for SourceAuthority {}

struct TargetAuthority;
impl AuthorityMarker for TargetAuthority {}
impl AuthorityProves<CanonicalOrder> for TargetAuthority {}

fn requires_target_authority(_: Proof<CanonicalOrder, TargetAuthority>) {}

fn main() {
    let source = AuthorityWitness::from_authority_marker(SourceAuthority);
    let source_scoped_proof = Proof::from_authority_witness(&source);

    requires_target_authority(source_scoped_proof);
}
// sealed-minting-case
