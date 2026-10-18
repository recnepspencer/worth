use std::marker::PhantomData;

use super::carrier::Artifact;
use crate::assumption::{
    AssumptionBasis, CurrentValidity, FreshnessScopedBasis, NoAssumptionBasis,
};
use crate::proof::{AuthorityMarker, AuthorityWitness, NoProofs, ProofSetAuthorizedBy};

impl<P, T> Artifact<P, T, NoProofs, NoAssumptionBasis> {
    pub fn new(payload: T) -> Self {
        Self {
            payload,
            proofs: NoProofs,
            basis: NoAssumptionBasis,
            phase: PhantomData,
        }
    }
}

impl<P, T, B> Artifact<P, T, NoProofs, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn with_current_basis<Auth>(
        payload: T,
        basis: B,
        _authority: AuthorityWitness<Auth>,
    ) -> Self
    where
        Auth: AuthorityMarker,
    {
        Self {
            payload,
            proofs: NoProofs,
            basis: FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
            phase: PhantomData,
        }
    }
}

impl<P, T, S, B> Artifact<P, T, S, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>>
where
    S: crate::proof::ProofSet,
{
    pub fn with_proofs_and_current_basis<Auth>(
        payload: T,
        proofs: S,
        basis: B,
        _authority: AuthorityWitness<Auth>,
    ) -> Self
    where
        Auth: AuthorityMarker,
        S: ProofSetAuthorizedBy<Auth>,
    {
        Self {
            payload,
            proofs,
            basis: FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
            phase: PhantomData,
        }
    }
}

impl<P, T, S, A> Artifact<P, T, S, A> {
    pub(crate) fn with_state(payload: T, proofs: S, basis: A) -> Self {
        Self {
            payload,
            proofs,
            basis,
            phase: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::artifact::{Artifact, ArtifactParts};
    use crate::assumption::NoAssumptionBasis;
    use crate::phase::PhaseMarker;
    use crate::proof::{mint_authority_witness, AuthorityMarker, NoProofs};

    struct RawPhase;
    impl PhaseMarker for RawPhase {}

    struct CurrentBasisAuthority;
    impl AuthorityMarker for CurrentBasisAuthority {}

    #[test]
    fn artifact_new_uses_empty_proof_and_basis_defaults() {
        let artifact = Artifact::<RawPhase, _>::new("payload");

        assert_eq!(artifact.payload(), &"payload");
        assert_eq!(artifact.proofs(), &NoProofs);
        assert_eq!(artifact.basis(), &NoAssumptionBasis);
    }

    #[test]
    fn artifact_current_basis_constructor_requires_authority_witness() {
        let artifact = Artifact::<RawPhase, _, _, _>::with_current_basis(
            "payload",
            13_u8,
            mint_authority_witness::<CurrentBasisAuthority>(),
        );

        assert_eq!(artifact.payload(), &"payload");
        assert_eq!(artifact.basis().basis().value(), &13_u8);
    }

    const _: () = assert!(
        size_of::<Artifact<RawPhase, u64, NoProofs, NoAssumptionBasis>>() == size_of::<u64>()
    );

    const _: () = assert!(
        size_of::<ArtifactParts<u64, NoProofs, NoAssumptionBasis>>()
            == size_of::<(u64, NoProofs, NoAssumptionBasis)>()
    );
}
