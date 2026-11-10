use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial, CanonicalDigestId,
    CanonicalDigestWorkBudget, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};

use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

pub(crate) struct InstallationCanonicalIdentityBasis {
    domain: CanonicalBasisDomain,
    version: CanonicalizationRuleVersion,
    budget: CanonicalDigestWorkBudget,
    entries: Vec<CanonicalBasisEntry>,
}

impl InstallationCanonicalIdentityBasis {
    pub(crate) fn new(
        domain_name: &'static str,
        version: &'static str,
        budget: CanonicalDigestWorkBudget,
    ) -> Self {
        Self {
            domain: CanonicalBasisDomain::Future(domain_name),
            version: CanonicalizationRuleVersion::new(version)
                .expect("fixed Query installation canonicalization rules are valid"),
            budget,
            entries: Vec::new(),
        }
    }

    pub(crate) fn text(
        &mut self,
        locus: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(locus, CanonicalBasisValue::ExactText(value.into().into()))
    }

    pub(crate) fn digest(
        &mut self,
        locus: impl Into<String>,
        value: CanonicalDigestId,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(locus, CanonicalBasisValue::BytesDigest(value))
    }

    pub(crate) fn unsigned_u32(
        &mut self,
        locus: impl Into<String>,
        value: u32,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits32,
                value: value.into(),
            },
        )
    }

    pub(crate) fn unsigned_usize(
        &mut self,
        locus: impl Into<String>,
        value: usize,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value as u128,
            },
        )
    }

    pub(crate) fn unsigned_u64(
        &mut self,
        locus: impl Into<String>,
        value: u64,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value.into(),
            },
        )
    }

    pub(crate) fn embedded_basis(
        &mut self,
        locus_prefix: &str,
        basis: &CanonicalBasisReadyArtifact,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        for entry in basis.payload().entries() {
            let CanonicalBasisLocus::Named(name) = entry.locus() else {
                unreachable!("declaration loci are named")
            };
            let name = name.raw_text().expect("declaration loci use raw text");
            self.push(format!("{locus_prefix}.{name}"), entry.value().clone())?;
        }
        Ok(())
    }

    pub(crate) fn derive(
        self,
    ) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial>
    {
        let basis = prepare_canonical_basis_sequence(self.version, self.domain, self.entries)
            .into_result()
            .expect("installation canonical identity bases are nonempty");
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), self.budget)
            .into_result()?;
        let derived = canonicalization().digest().derive(ready);
        Ok((
            CanonicalDigestId::new(*derived.value().bytes()),
            WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        ))
    }

    fn push(
        &mut self,
        locus: impl Into<String>,
        value: CanonicalBasisValue,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        let actual = u32::try_from(self.entries.len())
            .unwrap_or(u32::MAX)
            .saturating_add(1);
        if actual > self.budget.maximum_entry_count() {
            return Err(CanonicalDigestDerivationDenial::EntryLimitExceeded {
                maximum: self.budget.maximum_entry_count(),
                actual,
            });
        }
        self.entries.push(CanonicalBasisEntry::new(
            self.domain,
            CanonicalBasisLocus::Named(locus.into().into()),
            CanonicalBasisEntryKind::Field,
            value,
        ));
        Ok(())
    }
}
