use worth_proof::{
    Artifact, AuthorityMarker, AuthorityProves, AuthorityWitness, CanonicalOrder, Disjointness,
    PhaseMarker, Proof, ProofSetCons,
};

struct RawPhase;
impl PhaseMarker for RawPhase {}

struct FirstAuthority;
impl AuthorityMarker for FirstAuthority {}
impl AuthorityProves<CanonicalOrder> for FirstAuthority {}

struct SecondAuthority;
impl AuthorityMarker for SecondAuthority {}
impl AuthorityProves<Disjointness> for SecondAuthority {}

fn main() {
    let first = AuthorityWitness::from_authority_marker(FirstAuthority);
    let second = AuthorityWitness::from_authority_marker(SecondAuthority);
    let proofs = ProofSetCons::new(
        Proof::<CanonicalOrder, FirstAuthority>::from_authority_witness(&first),
        Proof::<Disjointness, SecondAuthority>::from_authority_witness(&second),
    );

    let _artifact = Artifact::<RawPhase, _, _, _>::with_proofs_and_current_basis(
        "payload", proofs, 7_u8, first,
    );
}
// sealed-minting-case
