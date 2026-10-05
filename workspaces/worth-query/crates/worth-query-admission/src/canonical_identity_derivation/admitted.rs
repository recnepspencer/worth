use super::{WorthQueryCanonicalIdentityBasis, WorthQueryCanonicalIdentityStop};
use crate::graph_read_access::digest_text::admitted_text_clone;
use worth_foundational::facade::{
    canonicalization, prepare_owned_canonical_basis_sequence_admitted, CanonicalBasisEntry,
    CanonicalBasisValue, CanonicalDigestAlgorithmId, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalIntegerWidth,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

pub(crate) struct WorthQueryAdmittedCanonicalIdentityBasis {
    basis: WorthQueryCanonicalIdentityBasis,
}

impl WorthQueryCanonicalIdentityBasis {
    pub(crate) fn new_admitted<Stop>(
        domain_name: &'static str,
        version: &'static str,
        budget: CanonicalDigestWorkBudget,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<WorthQueryAdmittedCanonicalIdentityBasis, WorthQueryCanonicalIdentityStop<Stop>>
    {
        use WorthQueryCanonicalIdentityStop::{AccountingOverflow, Admission};
        let bytes = u64::try_from(version.len()).map_err(|_| AccountingOverflow)?;
        // Rule-version construction owns the String copy, trim and whitespace
        // validation. The fixed domain remains a borrowed static string.
        let work = bytes
            .checked_mul(3)
            .and_then(|work| work.checked_add(3))
            .ok_or(AccountingOverflow)?;
        admit(work, bytes).map_err(Admission)?;
        Ok(WorthQueryAdmittedCanonicalIdentityBasis {
            basis: Self::new(domain_name, version, budget),
        })
    }
}

impl WorthQueryAdmittedCanonicalIdentityBasis {
    pub(crate) fn text_admitted<Stop>(
        &mut self,
        locus: &str,
        value: &str,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        self.reserve_entry_slot(admit)?;
        let locus = admitted_text_clone(locus, &mut *admit)?;
        let value = admitted_text_clone(value, &mut *admit)?;
        self.basis
            .append_value(locus, CanonicalBasisValue::ExactText(value.into()));
        Ok(())
    }

    /// Already-funded textual renderings move directly into canonical ownership.
    pub(crate) fn text_owned_admitted<Stop>(
        &mut self,
        locus: String,
        value: String,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        self.reserve_entry_slot(admit)?;
        self.basis
            .append_value(locus, CanonicalBasisValue::ExactText(value.into()));
        Ok(())
    }

    pub(crate) fn digest_admitted<Stop>(
        &mut self,
        locus: &str,
        value: CanonicalDigestId,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        self.reserve_entry_slot(admit)?;
        let locus = admitted_text_clone(locus, &mut *admit)?;
        self.basis
            .append_value(locus, CanonicalBasisValue::BytesDigest(value));
        Ok(())
    }

    pub(crate) fn unsigned_admitted<Stop>(
        &mut self,
        locus: &str,
        value: usize,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        let value = u64::try_from(value)
            .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        self.reserve_entry_slot(admit)?;
        let locus = admitted_text_clone(locus, &mut *admit)?;
        self.basis.append_value(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value.into(),
            },
        );
        Ok(())
    }

    pub(crate) fn unsigned_owned_admitted<Stop>(
        &mut self,
        locus: String,
        value: usize,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        let value = u64::try_from(value)
            .map_err(|_| WorthQueryCanonicalIdentityStop::AccountingOverflow)?;
        self.reserve_entry_slot(admit)?;
        self.basis.append_value(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value.into(),
            },
        );
        Ok(())
    }

    pub(crate) fn derive_admitted<Stop>(
        self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<
        (CanonicalDigestId, WorthQueryCanonicalWorkEvidence),
        WorthQueryCanonicalIdentityStop<Stop>,
    > {
        use WorthQueryCanonicalIdentityStop::{AccountingOverflow, Admission};
        let mut foundation_admit = |work: usize, bytes: usize| {
            let work = u64::try_from(work).map_err(|_| AccountingOverflow)?;
            let bytes = u64::try_from(bytes).map_err(|_| AccountingOverflow)?;
            admit(work, bytes).map_err(Admission)
        };
        let basis = prepare_owned_canonical_basis_sequence_admitted(
            self.basis.version,
            self.basis.domain,
            self.basis.entries,
            &mut foundation_admit,
        )?;
        // AlgorithmId::sha256 owns one six-byte String; pay before constructing it.
        foundation_admit(7, 6)?;
        let ready = canonicalization().digest().for_sequence_with_admission(
            basis,
            CanonicalDigestAlgorithmId::sha256(),
            self.basis.budget,
            &mut foundation_admit,
        )?;
        let derived = canonicalization()
            .digest()
            .derive_with_admission(ready, &mut foundation_admit)?;
        Ok((
            CanonicalDigestId::new(*derived.value().bytes()),
            WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        ))
    }

    fn reserve_entry_slot<Stop>(
        &mut self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<(), WorthQueryCanonicalIdentityStop<Stop>> {
        use WorthQueryCanonicalIdentityStop::{
            AccountingOverflow, Admission, AllocationUnavailable, Derivation,
        };
        admit(1, 0).map_err(Admission)?;
        self.basis.check_entry_limit().map_err(Derivation)?;
        let initialized = self.basis.entries.len();
        if initialized == self.basis.entries.capacity() {
            let requested = self
                .basis
                .entries
                .capacity()
                .checked_mul(2)
                .ok_or(AccountingOverflow)?
                .max(4)
                .min(self.basis.budget.maximum_entry_count() as usize);
            let bytes = requested
                .checked_mul(size_of::<CanonicalBasisEntry>())
                .ok_or(AccountingOverflow)?;
            let copies = initialized
                .checked_mul(size_of::<CanonicalBasisEntry>())
                .ok_or(AccountingOverflow)?;
            admit(
                u64::try_from(copies).map_err(|_| AccountingOverflow)?,
                u64::try_from(bytes).map_err(|_| AccountingOverflow)?,
            )
            .map_err(Admission)?;
            self.basis
                .entries
                .try_reserve_exact(requested - initialized)
                .map_err(|_| AllocationUnavailable)?;
        }
        // One initialized entry is written into its admitted slot.
        admit(size_of::<CanonicalBasisEntry>() as u64, 0).map_err(Admission)?;
        Ok(())
    }
}
