use worth_proof::{Artifact, AuthorityWitness, TransitionOutcome};

use crate::canonicalization::basis::{
    CanonicalBasisConstructionAuthority, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisReadyArtifact, CanonicalBundleReadinessProofs, CanonicalBundleReady,
    CanonicalizationRuleVersion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBasisBundle {
    version: CanonicalizationRuleVersion,
    sequences: Vec<CanonicalBasisReadyArtifact>,
}

impl CanonicalBasisBundle {
    pub(crate) fn new(
        version: CanonicalizationRuleVersion,
        sequences: Vec<CanonicalBasisReadyArtifact>,
    ) -> Self {
        Self { version, sequences }
    }

    pub(in crate::canonicalization) fn into_parts(
        self,
    ) -> (
        CanonicalizationRuleVersion,
        Vec<CanonicalBasisReadyArtifact>,
    ) {
        (self.version, self.sequences)
    }

    pub fn version(&self) -> &CanonicalizationRuleVersion {
        &self.version
    }

    pub fn sequences(&self) -> &[CanonicalBasisReadyArtifact] {
        &self.sequences
    }
}

pub type CanonicalBundleReadyArtifact = Artifact<
    CanonicalBundleReady,
    CanonicalBasisBundle,
    CanonicalBundleReadinessProofs,
    worth_proof::FreshnessScopedBasis<
        worth_proof::CurrentValidity,
        worth_proof::AssumptionBasis<CanonicalizationRuleVersion>,
    >,
>;

pub fn prepare_canonical_basis_bundle(
    version: CanonicalizationRuleVersion,
    sequences: impl IntoIterator<Item = CanonicalBasisReadyArtifact>,
) -> TransitionOutcome<CanonicalBundleReadyArtifact, CanonicalBasisConstructionDenial> {
    let mut sequences: Vec<_> = sequences.into_iter().collect();
    if sequences.is_empty() {
        return TransitionOutcome::denied(CanonicalBasisConstructionDenial::EmptySequence);
    }

    if sequences
        .iter()
        .any(|sequence| sequence.payload().version() != &version)
    {
        return TransitionOutcome::denied(
            CanonicalBasisConstructionDenial::BundleRuleVersionMismatch,
        );
    }

    sequences.sort_by_key(|sequence| sequence.payload().domain());
    if let Some(domain) = duplicate_domain(&sequences) {
        return TransitionOutcome::denied(
            CanonicalBasisConstructionDenial::DuplicateBundleDomain { domain },
        );
    }

    let bundle = CanonicalBasisBundle::new(version.clone(), sequences);
    let authority =
        AuthorityWitness::from_authority_marker(CanonicalBasisConstructionAuthority::new());
    let proofs = CanonicalBundleReadinessProofs::new(
        worth_proof::Proof::from_authority_witness(&authority),
        worth_proof::Proof::from_authority_witness(&authority),
    );

    TransitionOutcome::success(Artifact::with_proofs_and_current_basis(
        bundle, proofs, version, authority,
    ))
}

fn duplicate_domain(sequences: &[CanonicalBasisReadyArtifact]) -> Option<CanonicalBasisDomain> {
    sequences.windows(2).find_map(|window| {
        let left = window[0].payload().domain();
        let right = window[1].payload().domain();
        if left == right {
            Some(left)
        } else {
            None
        }
    })
}
