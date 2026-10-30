use worth_proof::{CanonicalOrder, Proof, StructuralProofAuthority};

fn takes_owned(_: Proof<CanonicalOrder, StructuralProofAuthority>) {}
fn requires_clone<T: Clone>(_: &T) {}

fn duplicate(proof: &Proof<CanonicalOrder, StructuralProofAuthority>) {
    requires_clone(proof);
    let _copied = *proof;
    takes_owned(*proof);
}

fn main() {}
// sealed-minting-case
