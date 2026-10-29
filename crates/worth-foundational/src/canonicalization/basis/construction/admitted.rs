use super::sequence::prepare_owned_sequence;
use crate::canonicalization::basis::{
    CanonicalBasisConstructionDenial, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisReadyArtifact, CanonicalizationRuleVersion,
};

/// Preparation denial preserves the original resource refusal and never emits
/// a Ready artifact after rejected work or allocation admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalBasisPreparationStop<Stop> {
    Construction(CanonicalBasisConstructionDenial),
    Resource {
        cause: Stop,
        quote: super::CanonicalBasisPreparationQuote,
    },
    AccountingOverflow,
    /// Stable-sort qualification refused for the actual configured compiler/target.
    UnsupportedSortImplementation(super::CanonicalSortImplementationDenial),
}

impl CanonicalBasisPreparationStop<()> {
    /// Puts the caller's original refusal back onto a stop whose cause was erased.
    pub(super) fn restore<Stop>(
        self,
        refusal: Option<Stop>,
    ) -> CanonicalBasisPreparationStop<Stop> {
        match self {
            Self::Construction(denial) => CanonicalBasisPreparationStop::Construction(denial),
            Self::Resource { quote, .. } => CanonicalBasisPreparationStop::Resource {
                cause: refusal.expect("resource callback retains its original refusal"),
                quote,
            },
            Self::AccountingOverflow => CanonicalBasisPreparationStop::AccountingOverflow,
            Self::UnsupportedSortImplementation(denial) => {
                CanonicalBasisPreparationStop::UnsupportedSortImplementation(denial)
            }
        }
    }
}

/// Moves owned entries through the ordinary canonical validation and sort.
/// Work and temporary storage are admitted by their owner before use; the
/// callback does not select canonical meaning or mint readiness proofs.
pub fn prepare_owned_canonical_basis_sequence_admitted<Stop>(
    version: CanonicalizationRuleVersion,
    domain: CanonicalBasisDomain,
    entries: Vec<CanonicalBasisEntry>,
    mut admit: impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<CanonicalBasisReadyArtifact, CanonicalBasisPreparationStop<Stop>> {
    super::sort_qualification::require_qualified_sort()
        .map_err(CanonicalBasisPreparationStop::UnsupportedSortImplementation)?;
    prepare_owned_sequence(version, domain, entries, Some(&mut admit))
}
