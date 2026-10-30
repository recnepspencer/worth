use worth_proof::{
    AuthorityMarker, AuthorityProves, AuthorityWitness, CanonicalOrder, Disjointness, Proof,
};

struct NarrowAuthority;
impl AuthorityMarker for NarrowAuthority {}
impl AuthorityProves<CanonicalOrder> for NarrowAuthority {}

fn main() {
    let authority = AuthorityWitness::from_authority_marker(NarrowAuthority);
    let _proof = Proof::<Disjointness, NarrowAuthority>::from_authority_witness(&authority);
}
// sealed-minting-case
