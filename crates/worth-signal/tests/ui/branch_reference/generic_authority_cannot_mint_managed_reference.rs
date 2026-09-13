use worth_proof::{AuthorityMarker, AuthorityWitness};
use worth_signal::facade::branch::ManagedSignalBranchReference;

fn generic_door<Auth: AuthorityMarker>(
    authority: AuthorityWitness<Auth>,
) -> ManagedSignalBranchReference {
    authority.into()
}

fn valid_generic_carriage<Auth: AuthorityMarker>(
    authority: AuthorityWitness<Auth>,
) -> AuthorityWitness<Auth> {
    authority
}

fn main() {}
