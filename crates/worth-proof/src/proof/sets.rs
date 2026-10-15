use std::marker::PhantomData;

use super::markers::ProofMarker;
use super::witnesses::{AuthorityMarker, AuthorityWitness};

pub trait ProofSet: 'static {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoProofs;

impl ProofSet for NoProofs {}

pub trait AuthorityProves<P>: AuthorityMarker
where
    P: ProofMarker,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proof<P, A>(PhantomData<(P, A)>);

impl<P, A> Proof<P, A> {
    /// Crate-internal. Reached only from a checked constructor that has
    /// already established the fact `P` names.
    pub(crate) fn mint() -> Self {
        Self(PhantomData)
    }
}

impl<P, A> Proof<P, A>
where
    P: ProofMarker,
    A: AuthorityProves<P>,
{
    pub fn from_authority_witness(_authority: &AuthorityWitness<A>) -> Self
    where
        A: AuthorityMarker,
    {
        Self(PhantomData)
    }
}

impl<P, A> ProofSet for Proof<P, A>
where
    P: ProofMarker,
    A: AuthorityMarker,
{
}

pub trait ProofSetAuthorizedBy<Auth>: ProofSet
where
    Auth: AuthorityMarker,
{
}

impl<Auth> ProofSetAuthorizedBy<Auth> for NoProofs where Auth: AuthorityMarker {}

impl<P, Auth> ProofSetAuthorizedBy<Auth> for Proof<P, Auth>
where
    P: ProofMarker,
    Auth: AuthorityProves<P>,
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProofSetCons<Head, Tail> {
    head: Head,
    tail: Tail,
}

impl<Head, Tail> ProofSetCons<Head, Tail> {
    pub fn new(head: Head, tail: Tail) -> Self {
        Self { head, tail }
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn tail(&self) -> &Tail {
        &self.tail
    }
}

impl<Head, Tail> ProofSet for ProofSetCons<Head, Tail>
where
    Head: ProofSet,
    Tail: ProofSet,
{
}

impl<Head, Tail, Auth> ProofSetAuthorizedBy<Auth> for ProofSetCons<Head, Tail>
where
    Head: ProofSetAuthorizedBy<Auth>,
    Tail: ProofSetAuthorizedBy<Auth>,
    Auth: AuthorityMarker,
{
}

#[cfg(test)]
mod tests {
    use super::{NoProofs, ProofSet, ProofSetCons};
    use crate::proof::{mint_proof, AuthorityMarker, AuthorityProves, ProofMarker};

    struct LeftProof;
    impl ProofMarker for LeftProof {}

    struct RightProof;
    impl ProofMarker for RightProof {}

    #[derive(Debug, PartialEq, Eq)]
    struct TestAuthority;
    impl AuthorityMarker for TestAuthority {}
    impl AuthorityProves<LeftProof> for TestAuthority {}
    impl AuthorityProves<RightProof> for TestAuthority {}

    fn accepts_proof_set<T: ProofSet>(_: &T) {}

    #[test]
    fn nested_proof_sets_preserve_head_and_tail_access() {
        let proofs = ProofSetCons::new(
            mint_proof::<LeftProof, TestAuthority>(),
            ProofSetCons::new(mint_proof::<RightProof, TestAuthority>(), NoProofs),
        );

        accepts_proof_set(&proofs);
        accepts_proof_set(proofs.tail());
        assert_eq!(*proofs.tail().tail(), NoProofs);
    }
}
