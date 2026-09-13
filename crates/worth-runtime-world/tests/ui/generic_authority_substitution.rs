use worth_proof::AuthorityWitness;
use worth_runtime_world::facade::{
    AdmittedRelationalBranchBasis, AdmittedRuntimeWorldCorrespondenceBasis,
    AdmittedSignalBranchBasis, ProductBranchCreationIntent, RuntimeWorldBootstrapIntent,
};

fn generic_authority_cannot_replace_concrete_correspondence<Auth: worth_proof::AuthorityMarker>(
    creation: ProductBranchCreationIntent,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    authority: AuthorityWitness<Auth>,
) {
    let _ = RuntimeWorldBootstrapIntent::new(creation, relational, signal, authority);
}

fn concrete_correspondence_is_the_required_proof(
    creation: ProductBranchCreationIntent,
    relational: AdmittedRelationalBranchBasis,
    signal: AdmittedSignalBranchBasis,
    correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
) {
    let _ = RuntimeWorldBootstrapIntent::new(creation, relational, signal, correspondence);
}

fn main() {}
