//! Identity-local exact contract interning, in canonical scope first-use order.
use std::collections::BTreeMap;

use worth_foundational::facade::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisReadyArtifact,
    CanonicalDigestDerivationDenial,
};

use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;

#[derive(Default)]
pub(super) struct AspectContractTable<'a> {
    sources: BTreeMap<*const CanonicalBasisReadyArtifact, usize>,
    slots: BTreeMap<&'a [CanonicalBasisEntry], usize>,
    contracts: Vec<&'a CanonicalBasisReadyArtifact>,
}

impl<'a> AspectContractTable<'a> {
    /// Reuses carried canonical entries without re-encoding or hashing them.
    /// Every scope pays one reference entry; a new contract pays its complete
    /// basis plus version/count framing before entering the table. Thus both
    /// table breadth and comparisons are bounded by the same identity budget.
    /// Installed scopes share catalog allocations. A borrowed pointer lookup
    /// carries that sameness without rescanning fields on every write; pointers
    /// never enter canonical material. Separate allocations use exact entries.
    pub(super) fn encode_reference(
        &mut self,
        basis: &mut InstallationCanonicalIdentityBasis,
        locus: &str,
        contract: &'a CanonicalBasisReadyArtifact,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        if contract.payload().domain() != CanonicalBasisDomain::AspectContract {
            return Err(CanonicalDigestDerivationDenial::InputDomainMismatch);
        }
        let entries = contract.payload().entries();
        let source = std::ptr::from_ref(contract);
        let slot = if let Some(slot) = self.sources.get(&source) {
            *slot
        } else if let Some(slot) = self.slots.get(entries) {
            *slot
        } else {
            let slot = self.contracts.len();
            let prefix = format!("aspect-contract[{slot}]");
            basis.text(
                format!("{prefix}.version"),
                contract.payload().version().as_str(),
            )?;
            basis.unsigned_usize(format!("{prefix}.entry-count"), entries.len())?;
            basis.embedded_basis(&format!("{prefix}.basis"), contract)?;
            self.contracts.push(contract);
            self.slots.insert(entries, slot);
            slot
        };
        self.validate_reference(slot, contract)?;
        basis.unsigned_usize(locus, slot)?;
        self.sources.insert(source, slot);
        Ok(())
    }

    fn validate_reference(
        &self,
        slot: usize,
        expected: &CanonicalBasisReadyArtifact,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        let Some(actual) = self.contracts.get(slot) else {
            return Err(CanonicalDigestDerivationDenial::InputShapeMismatch);
        };
        if std::ptr::eq(*actual, expected) {
            return Ok(());
        }
        if actual.payload().version() != expected.payload().version()
            || actual.payload().entries() != expected.payload().entries()
        {
            return Err(CanonicalDigestDerivationDenial::InputShapeMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
