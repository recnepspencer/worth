#[cfg(test)]
use super::{markers::ProofMarker, sets::AuthorityProves, sets::Proof};

/// Test-only convenience over [`Proof::mint`]. Production proof creation
/// goes through a checked constructor that validates before minting.
#[cfg(test)]
pub(crate) fn mint_proof<P, Auth>() -> Proof<P, Auth>
where
    P: ProofMarker,
    Auth: AuthorityProves<P>,
{
    Proof::mint()
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::proof::{AuthorityMarker, AuthorityProves, CanonicalOrder, Proof};

    struct TestAuthority;
    impl AuthorityMarker for TestAuthority {}
    impl AuthorityProves<CanonicalOrder> for TestAuthority {}

    const _: () = assert!(size_of::<Proof<CanonicalOrder, TestAuthority>>() == 0);
}
