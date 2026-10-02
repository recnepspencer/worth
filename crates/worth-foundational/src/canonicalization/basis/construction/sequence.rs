use worth_proof::{Artifact, AuthorityWitness, TransitionOutcome};

use crate::canonicalization::basis::{
    CanonicalBasisConstructionAuthority, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisReadinessProofs, CanonicalBasisReady, CanonicalBasisValue,
    CanonicalizationCost, CanonicalizationRuleVersion,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalBasisSequence {
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: Vec<CanonicalBasisEntry>,
    cost: CanonicalizationCost,
}

impl CanonicalBasisSequence {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.version
            .owned_allocation_capacity_bytes()
            .saturating_add(
                self.entries
                    .capacity()
                    .saturating_mul(std::mem::size_of::<CanonicalBasisEntry>()),
            )
            .saturating_add(
                self.entries
                    .iter()
                    .map(CanonicalBasisEntry::owned_allocation_capacity_bytes)
                    .sum::<usize>(),
            )
    }

    pub(crate) fn new(
        version: CanonicalizationRuleVersion,
        domain: CanonicalBasisDomain,
        entries: Vec<CanonicalBasisEntry>,
        cost: CanonicalizationCost,
    ) -> Self {
        Self {
            version,
            domain,
            entries,
            cost,
        }
    }

    pub fn version(&self) -> &CanonicalizationRuleVersion {
        &self.version
    }

    pub const fn domain(&self) -> CanonicalBasisDomain {
        self.domain
    }

    pub fn entries(&self) -> &[CanonicalBasisEntry] {
        &self.entries
    }

    pub const fn cost(&self) -> CanonicalizationCost {
        self.cost
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        CanonicalizationRuleVersion,
        CanonicalBasisDomain,
        Vec<CanonicalBasisEntry>,
        CanonicalizationCost,
    ) {
        (self.version, self.domain, self.entries, self.cost)
    }
}

pub type CanonicalBasisReadyArtifact = Artifact<
    CanonicalBasisReady,
    CanonicalBasisSequence,
    CanonicalBasisReadinessProofs,
    worth_proof::FreshnessScopedBasis<
        worth_proof::CurrentValidity,
        worth_proof::AssumptionBasis<CanonicalizationRuleVersion>,
    >,
>;

pub fn prepare_canonical_basis_sequence(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: impl IntoIterator<Item = CanonicalBasisEntry>,
) -> TransitionOutcome<CanonicalBasisReadyArtifact, CanonicalBasisConstructionDenial> {
    let mut entries: Vec<_> = entries.into_iter().collect();
    if entries.is_empty() {
        return TransitionOutcome::denied(CanonicalBasisConstructionDenial::EmptySequence);
    }

    if let Some(entry) = entries.iter().find(|entry| entry.domain() != domain) {
        return TransitionOutcome::denied(CanonicalBasisConstructionDenial::DomainMismatch {
            expected: domain,
            actual: entry.domain(),
        });
    }

    let nested_sequence_count = entries
        .iter()
        .filter(|entry| matches!(entry.value(), CanonicalBasisValue::NestedSequence(_)))
        .count() as u32;
    let compatibility_lowering_count = entries
        .iter()
        .filter(|entry| entry.domain() == CanonicalBasisDomain::CompatibilityLowering)
        .count() as u32;

    let mut ordering_comparisons = 0_u32;
    entries.sort_by(|left, right| {
        ordering_comparisons = ordering_comparisons.saturating_add(1);
        left.cmp(right)
    });

    if let Some(duplicate) = entries.windows(2).find_map(|window| {
        let left = &window[0];
        let right = &window[1];
        if left.domain() == right.domain()
            && left.locus() == right.locus()
            && left.kind() == right.kind()
        {
            Some((left.domain(), left.locus().clone(), left.kind()))
        } else {
            None
        }
    }) {
        return TransitionOutcome::denied(CanonicalBasisConstructionDenial::DuplicateEntry {
            domain: duplicate.0,
            locus: duplicate.1,
            kind: duplicate.2,
        });
    }

    let cost = CanonicalizationCost::new(
        entries.len() as u32,
        ordering_comparisons,
        nested_sequence_count,
        compatibility_lowering_count,
    );
    let sequence = CanonicalBasisSequence::new(version.clone(), domain, entries, cost);
    let authority =
        AuthorityWitness::from_authority_marker(CanonicalBasisConstructionAuthority::new());
    let proofs = CanonicalBasisReadinessProofs::new(
        worth_proof::Proof::from_authority_witness(&authority),
        worth_proof::ProofSetCons::new(
            worth_proof::Proof::from_authority_witness(&authority),
            worth_proof::ProofSetCons::new(
                worth_proof::Proof::from_authority_witness(&authority),
                worth_proof::ProofSetCons::new(
                    worth_proof::Proof::from_authority_witness(&authority),
                    worth_proof::Proof::from_authority_witness(&authority),
                ),
            ),
        ),
    );

    TransitionOutcome::success(Artifact::with_proofs_and_current_basis(
        sequence, proofs, version, authority,
    ))
}
