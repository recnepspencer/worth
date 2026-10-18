use crate::artifact::Artifact;

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinInputs2<L, R> {
    left: L,
    right: R,
}

pub type ArtifactJoinInputs2<P, L, R, LS, RS, LA, RA> =
    JoinInputs2<Artifact<P, L, LS, LA>, Artifact<P, R, RS, RA>>;

impl<L, R> JoinInputs2<L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }

    pub fn left(&self) -> &L {
        &self.left
    }

    pub fn right(&self) -> &R {
        &self.right
    }

    pub fn into_parts(self) -> (L, R) {
        (self.left, self.right)
    }
}

pub fn join_artifact_pair<P, L, R, LS, RS, LA, RA, T, S, A>(
    inputs: ArtifactJoinInputs2<P, L, R, LS, RS, LA, RA>,
    join: impl FnOnce((L, LS, LA), (R, RS, RA)) -> (T, S, A),
) -> Artifact<P, T, S, A> {
    let (left, right) = inputs.into_parts();
    let left = left.into_parts().into_parts();
    let right = right.into_parts().into_parts();
    let (payload, proofs, basis) = join(left, right);

    Artifact::with_state(payload, proofs, basis)
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::artifact::Artifact;
    use crate::assumption::{AssumptionBasis, NoAssumptionBasis};
    use crate::phase::PhaseMarker;
    use crate::proof::{
        mint_proof, AuthorityMarker, AuthorityProves, NoProofs, Proof, ProofMarker,
    };

    use super::{join_artifact_pair, JoinInputs2};

    struct LoweredPhase;
    impl PhaseMarker for LoweredPhase {}

    struct JoinedProof;
    impl ProofMarker for JoinedProof {}

    struct JoinAuthority;
    impl AuthorityMarker for JoinAuthority {}
    impl AuthorityProves<JoinedProof> for JoinAuthority {}

    #[test]
    fn artifact_join_requires_explicit_output_state() {
        let left = Artifact::<LoweredPhase, _, _, _>::with_state(
            3_u8,
            NoProofs,
            AssumptionBasis::new(4_u8),
        );
        let right = Artifact::<LoweredPhase, _, _, _>::with_state(
            5_u8,
            mint_proof::<JoinedProof, JoinAuthority>(),
            NoAssumptionBasis,
        );

        let joined = join_artifact_pair(JoinInputs2::new(left, right), |left, right| {
            let (left_payload, _left_proofs, left_basis) = left;
            let (right_payload, right_proofs, _right_basis) = right;

            (
                left_payload + right_payload + left_basis.value(),
                right_proofs,
                NoAssumptionBasis,
            )
        });

        assert_eq!(joined.payload(), &12_u8);
        let _: &Proof<JoinedProof, JoinAuthority> = joined.proofs();
    }

    const _: () = assert!(size_of::<JoinInputs2<u64, u16>>() == size_of::<(u64, u16)>());
}
