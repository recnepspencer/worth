use std::marker::PhantomData;

use crate::artifact::Artifact;
use crate::proof::{AuthorityMarker, AuthorityWitness};
use crate::recipe::{Admitted, Lowered, Recipe, Resolved};

use super::{
    AssumptionBasis, AuthorityRevalidationRequiredBasis, CurrentValidity, FreshnessScopedBasis,
    RebindRequiredBasis, StaleReadableBasis,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryBridged<A> {
    weakened_basis: A,
    boundary: PhantomData<fn() -> ()>,
}

impl<A> BoundaryBridged<A> {
    pub(crate) fn new(weakened_basis: A) -> Self {
        Self {
            weakened_basis,
            boundary: PhantomData,
        }
    }

    pub fn weakened_basis(&self) -> &A {
        &self.weakened_basis
    }

    pub fn into_weakened_basis(self) -> A {
        self.weakened_basis
    }
}

pub type BoundaryBridgedStaleReadableBasis<B> = BoundaryBridged<StaleReadableBasis<B>>;
pub type BoundaryBridgedRebindRequiredBasis<B> = BoundaryBridged<RebindRequiredBasis<B>>;
pub type BoundaryBridgedAuthorityRevalidationRequiredBasis<B> =
    BoundaryBridged<AuthorityRevalidationRequiredBasis<B>>;

impl<P, T, S, B> Artifact<P, T, S, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn bridge_trust_boundary(
        self,
    ) -> Artifact<P, T, S, BoundaryBridgedAuthorityRevalidationRequiredBasis<B>> {
        Artifact::with_state(
            self.payload,
            self.proofs,
            BoundaryBridged::new(AuthorityRevalidationRequiredBasis::new(
                self.basis.into_basis(),
            )),
        )
    }
}

impl<P, T, S, B> Artifact<P, T, S, BoundaryBridgedAuthorityRevalidationRequiredBasis<B>> {
    pub fn readmit_with_authority<Auth, NextB>(
        self,
        basis: NextB,
        _authority: AuthorityWitness<Auth>,
    ) -> Artifact<P, T, S, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<NextB>>>
    where
        Auth: AuthorityMarker,
    {
        Artifact::with_state(
            self.payload,
            self.proofs,
            FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
        )
    }
}

impl<T, B> Recipe<Resolved, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn bridge_trust_boundary(
        self,
    ) -> Recipe<Resolved, T, BoundaryBridgedRebindRequiredBasis<B>> {
        Recipe::with_stage(
            self.payload,
            BoundaryBridged::new(RebindRequiredBasis::new(self.basis.into_basis())),
        )
    }
}

impl<T, B> Recipe<Resolved, T, BoundaryBridgedRebindRequiredBasis<B>> {
    pub fn rebind_with_authority<Auth, NextB>(
        self,
        basis: NextB,
        _authority: AuthorityWitness<Auth>,
    ) -> Recipe<Resolved, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<NextB>>>
    where
        Auth: AuthorityMarker,
    {
        Recipe::with_stage(
            self.payload,
            FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
        )
    }
}

impl<T, B> Recipe<Lowered, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn bridge_trust_boundary(self) -> Recipe<Lowered, T, BoundaryBridgedStaleReadableBasis<B>> {
        Recipe::with_stage(
            self.payload,
            BoundaryBridged::new(StaleReadableBasis::new(self.basis.into_basis())),
        )
    }
}

impl<T, B> Recipe<Lowered, T, BoundaryBridgedStaleReadableBasis<B>> {
    pub fn readmit_with_authority<Auth, NextB>(
        self,
        basis: NextB,
        _authority: AuthorityWitness<Auth>,
    ) -> Recipe<Lowered, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<NextB>>>
    where
        Auth: AuthorityMarker,
    {
        Recipe::with_stage(
            self.payload,
            FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
        )
    }
}

impl<T, B> Recipe<Admitted, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<B>>> {
    pub fn bridge_trust_boundary(
        self,
    ) -> Recipe<Admitted, T, BoundaryBridgedAuthorityRevalidationRequiredBasis<B>> {
        Recipe::with_stage(
            self.payload,
            BoundaryBridged::new(AuthorityRevalidationRequiredBasis::new(
                self.basis.into_basis(),
            )),
        )
    }
}

impl<T, B> Recipe<Admitted, T, BoundaryBridgedAuthorityRevalidationRequiredBasis<B>> {
    pub fn readmit_with_authority<Auth, NextB>(
        self,
        basis: NextB,
        _authority: AuthorityWitness<Auth>,
    ) -> Recipe<Admitted, T, FreshnessScopedBasis<CurrentValidity, AssumptionBasis<NextB>>>
    where
        Auth: AuthorityMarker,
    {
        Recipe::with_stage(
            self.payload,
            FreshnessScopedBasis::new(AssumptionBasis::new(basis)),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::artifact::Artifact;
    use crate::assumption::{
        AssumptionBasis, BoundaryBridged, BoundaryBridgedAuthorityRevalidationRequiredBasis,
        BoundaryBridgedRebindRequiredBasis, BoundaryBridgedStaleReadableBasis, CurrentValidity,
        FreshnessScopedBasis,
    };
    use crate::phase::PhaseMarker;
    use crate::proof::{
        mint_authority_witness, mint_proof, AuthorityMarker, CanonicalOrder,
        StructuralProofAuthority,
    };
    use crate::recipe::{Admitted, Lowered, Recipe, Resolved};

    struct ValidatedPhase;
    impl PhaseMarker for ValidatedPhase {}

    struct ReadmissionAuthority;
    impl AuthorityMarker for ReadmissionAuthority {}

    const _: () = assert!(
        size_of::<BoundaryBridged<AssumptionBasis<u8>>>() == size_of::<AssumptionBasis<u8>>()
    );

    #[test]
    fn artifact_trust_boundary_bridge_and_readmission_preserve_payload_and_proofs() {
        let artifact = Artifact::<ValidatedPhase, _, _, _>::with_state(
            "payload",
            mint_proof::<CanonicalOrder, StructuralProofAuthority>(),
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(7_u8)),
        );

        let bridged: Artifact<
            ValidatedPhase,
            _,
            _,
            BoundaryBridgedAuthorityRevalidationRequiredBasis<u8>,
        > = artifact.bridge_trust_boundary();
        assert_eq!(bridged.basis().weakened_basis().basis().value(), &7_u8);

        let readmitted = bridged
            .readmit_with_authority(11_u16, mint_authority_witness::<ReadmissionAuthority>());
        assert_eq!(readmitted.payload(), &"payload");
        assert_eq!(readmitted.strong_basis().value(), &11_u16);
    }

    #[test]
    fn recipe_trust_boundary_bridge_distinguishes_rebind_stale_and_authority_paths() {
        let resolved = Recipe::<Resolved, _, _>::with_stage(
            "resolved",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(13_u8)),
        );
        let lowered = Recipe::<Lowered, _, _>::with_stage(
            "lowered",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(17_u8)),
        );
        let admitted = Recipe::<Admitted, _, _>::with_stage(
            "admitted",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(19_u8)),
        );

        let bridged_resolved: Recipe<Resolved, _, BoundaryBridgedRebindRequiredBasis<u8>> =
            resolved.bridge_trust_boundary();
        let bridged_lowered: Recipe<Lowered, _, BoundaryBridgedStaleReadableBasis<u8>> =
            lowered.bridge_trust_boundary();
        let bridged_admitted: Recipe<
            Admitted,
            _,
            BoundaryBridgedAuthorityRevalidationRequiredBasis<u8>,
        > = admitted.bridge_trust_boundary();

        assert_eq!(
            bridged_resolved.basis().weakened_basis().basis().value(),
            &13_u8
        );
        assert_eq!(
            bridged_lowered.basis().weakened_basis().basis().value(),
            &17_u8
        );
        assert_eq!(
            bridged_admitted.basis().weakened_basis().basis().value(),
            &19_u8
        );
    }

    #[test]
    fn recipe_readmission_requires_explicit_rebind_or_authority_progression() {
        let resolved = Recipe::<Resolved, _, _>::with_stage(
            "resolved",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(23_u8)),
        );
        let lowered = Recipe::<Lowered, _, _>::with_stage(
            "lowered",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(27_u8)),
        );
        let admitted = Recipe::<Admitted, _, _>::with_stage(
            "admitted",
            FreshnessScopedBasis::<CurrentValidity, _>::new(AssumptionBasis::new(29_u8)),
        );

        let rebound = resolved
            .bridge_trust_boundary()
            .rebind_with_authority(31_u16, mint_authority_witness::<ReadmissionAuthority>());
        let revalidated = lowered
            .bridge_trust_boundary()
            .readmit_with_authority(35_u16, mint_authority_witness::<ReadmissionAuthority>());
        let readmitted = admitted
            .bridge_trust_boundary()
            .readmit_with_authority(37_u16, mint_authority_witness::<ReadmissionAuthority>());

        assert_eq!(rebound.strong_basis().value(), &31_u16);
        assert_eq!(revalidated.strong_basis().value(), &35_u16);
        assert_eq!(readmitted.strong_basis().value(), &37_u16);
    }
}
