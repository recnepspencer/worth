use worth_proof::AuthorityWitness;
use worth_signal::facade::SignalBranchBasisReadmissionAuthority;

fn main() {
    let _authority = AuthorityWitness::from_authority_marker(
        SignalBranchBasisReadmissionAuthority::new(),
    );
}
