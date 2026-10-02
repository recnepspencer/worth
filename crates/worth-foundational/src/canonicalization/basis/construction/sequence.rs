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
    match prepare_owned_sequence::<std::convert::Infallible>(
        version,
        domain,
        entries.into_iter().collect(),
        None,
    ) {
        Ok(ready) => TransitionOutcome::success(ready),
        Err(super::CanonicalBasisPreparationStop::Construction(denial)) => {
            TransitionOutcome::denied(denial)
        }
        Err(super::CanonicalBasisPreparationStop::Resource(impossible)) => match impossible {},
        Err(super::CanonicalBasisPreparationStop::AccountingOverflow) => {
            unreachable!("ordinary preparation performs no resource arithmetic")
        }
        Err(super::CanonicalBasisPreparationStop::UnsupportedSortImplementation) => {
            unreachable!("ordinary preparation has no admitted sort restriction")
        }
    }
}

pub(super) fn prepare_owned_sequence<Stop>(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    mut entries: Vec<CanonicalBasisEntry>,
    mut admit: Option<&mut dyn FnMut(usize, usize) -> Result<(), Stop>>,
) -> Result<CanonicalBasisReadyArtifact, super::CanonicalBasisPreparationStop<Stop>> {
    use super::CanonicalBasisPreparationStop::{AccountingOverflow, Construction, Resource};
    if let Some(admit) = admit.as_mut() {
        u32::try_from(entries.len()).map_err(|_| AccountingOverflow)?;
        let domain_width = match domain {
            CanonicalBasisDomain::Future(name) => name.len(),
            _ => 0,
        };
        let visit_work = domain_width
            .checked_mul(2)
            .and_then(|work| work.checked_add(3))
            .and_then(|work| work.checked_mul(entries.len()))
            .and_then(|work| work.checked_add(1))
            .ok_or(AccountingOverflow)?;
        admit(visit_work, 0).map_err(Resource)?;
    }
    if entries.is_empty() {
        return Err(Construction(
            CanonicalBasisConstructionDenial::EmptySequence,
        ));
    }

    if let Some(entry) = entries.iter().find(|entry| entry.domain() != domain) {
        return Err(Construction(
            CanonicalBasisConstructionDenial::DomainMismatch {
                expected: domain,
                actual: entry.domain(),
            },
        ));
    }

    let nested_sequence_count = entries
        .iter()
        .filter(|entry| matches!(entry.value(), CanonicalBasisValue::NestedSequence(_)))
        .count() as u32;
    let compatibility_lowering_count = entries
        .iter()
        .filter(|entry| entry.domain() == CanonicalBasisDomain::CompatibilityLowering)
        .count() as u32;

    if let Some(admit) = admit.as_mut() {
        let width = super::entry_measurement::measure(&entries, admit)?;
        let (work, bytes) =
            super::sort_admission::sorting_claim(entries.len(), width).ok_or(AccountingOverflow)?;
        let version_bytes = version.as_str().len();
        admit(
            work.checked_add(version_bytes)
                .and_then(|work| work.checked_add(1))
                .ok_or(AccountingOverflow)?,
            bytes.checked_add(version_bytes).ok_or(AccountingOverflow)?,
        )
        .map_err(Resource)?;
    }

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
        return Err(Construction(
            CanonicalBasisConstructionDenial::DuplicateEntry {
                domain: duplicate.0,
                locus: duplicate.1,
                kind: duplicate.2,
            },
        ));
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

    Ok(Artifact::with_proofs_and_current_basis(
        sequence, proofs, version, authority,
    ))
}
