use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

mod admission_stop;
mod admitted;
pub use admission_stop::WorthQueryCanonicalIdentityStop;

pub(crate) struct WorthQueryCanonicalIdentityBasis {
    domain: CanonicalBasisDomain,
    version: CanonicalizationRuleVersion,
    budget: CanonicalDigestWorkBudget,
    entries: Vec<CanonicalBasisEntry>,
}

impl WorthQueryCanonicalIdentityBasis {
    pub(crate) fn new(
        domain_name: &'static str,
        version: &'static str,
        budget: CanonicalDigestWorkBudget,
    ) -> Self {
        Self {
            domain: CanonicalBasisDomain::Future(domain_name),
            version: CanonicalizationRuleVersion::new(version)
                .expect("fixed Query canonicalization rules are valid"),
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

    pub(crate) fn unsigned(
        &mut self,
        locus: impl Into<String>,
        value: usize,
    ) -> Result<(), CanonicalDigestDerivationDenial> {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: u64::try_from(value)
                    .expect("Query canonical structural counts fit u64")
                    .into(),
            },
        )
    }

    pub(crate) fn derive(
        self,
    ) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial>
    {
        let basis = prepare_canonical_basis_sequence(self.version, self.domain, self.entries)
            .into_result()
            .expect("bounded Query canonical bases are nonempty");
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
        self.check_entry_limit()?;
        self.append_value(locus.into(), value);
        Ok(())
    }

    fn check_entry_limit(&self) -> Result<(), CanonicalDigestDerivationDenial> {
        let actual = u32::try_from(self.entries.len())
            .unwrap_or(u32::MAX)
            .saturating_add(1);
        if actual > self.budget.maximum_entry_count() {
            return Err(CanonicalDigestDerivationDenial::EntryLimitExceeded {
                maximum: self.budget.maximum_entry_count(),
                actual,
            });
        }
        Ok(())
    }

    fn append_value(&mut self, locus: String, value: CanonicalBasisValue) {
        self.entries.push(CanonicalBasisEntry::new(
            self.domain,
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Field,
            value,
        ));
    }
}
