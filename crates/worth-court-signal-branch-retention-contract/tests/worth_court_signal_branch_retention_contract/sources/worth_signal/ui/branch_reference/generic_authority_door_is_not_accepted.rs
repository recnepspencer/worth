use worth_proof::{AuthorityMarker, AuthorityWitness};
use worth_signal::facade::branch::SignalBranchBasisAuthority;

fn generic_door<Auth: AuthorityMarker>(
    authority: AuthorityWitness<Auth>,
) {
    let _owner_authority: SignalBranchBasisAuthority = authority;
}

fn main() {}
