use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact, CanonicalBasisValue,
    CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial, CanonicalDigestId,
    CanonicalDigestWorkBudget, CanonicalizationRuleVersion,
};
use worth_query_declaration::facade::application_capability::{
    application_capability_canonical_components, ErasedApplicationCapabilityContract,
};

use super::{
    WorthQueryApplicationCapabilityInstallationDenial,
    WorthQueryApplicationCapabilityInstallationDenialKind,
};
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-capability-installation");
const RULE_VERSION: &str = "worth-query-application-capability-installation-v4";
const MAXIMUM_ENTRY_COUNT: u32 = 512;
const MAXIMUM_CANONICAL_BYTES: usize = 128 * 1_024;
const CAPABILITY_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(MAXIMUM_ENTRY_COUNT, MAXIMUM_CANONICAL_BYTES) {
        Some(budget) => budget,
        None => panic!("fixed capability canonical-work budget is valid"),
    };

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCapabilityCanonicalArtifact {
    basis: CanonicalBasisReadyArtifact,
    digest: CanonicalDigestId,
    work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryCapabilityCanonicalArtifact {
    pub fn basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn canonical_encoded_bytes(&self) -> usize {
        self.work.canonical_encoded_bytes()
    }

    pub const fn canonical_entry_count(&self) -> u32 {
        self.work.canonical_entries()
    }

    pub const fn maximum_canonical_encoded_bytes(&self) -> usize {
        MAXIMUM_CANONICAL_BYTES
    }

    pub const fn maximum_canonical_entry_count(&self) -> u32 {
        MAXIMUM_ENTRY_COUNT
    }

    pub const fn basis_preparation_count(&self) -> usize {
        self.work.basis_preparations() as usize
    }

    pub const fn digest_derivation_count(&self) -> usize {
        self.work.digest_derivations() as usize
    }

    pub const fn work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.work
    }
}

pub(super) fn prepare_capability_basis(
    package_identity: &CanonicalDigestId,
    schema_identity: &CanonicalDigestId,
    contract: &ErasedApplicationCapabilityContract,
) -> Result<WorthQueryCapabilityCanonicalArtifact, WorthQueryApplicationCapabilityInstallationDenial>
{
    let mut builder = CapabilityBasisBuilder::new(contract.name());
    builder.text("family", "installed-capability");
    builder.digest("package", package_identity);
    builder.digest("schema", schema_identity);
    for component in application_capability_canonical_components(contract) {
        builder.value(component.locus(), component.value().clone());
    }
    builder.finish()
}

struct CapabilityBasisBuilder {
    subject: String,
    entries: Vec<CanonicalBasisEntry>,
}

impl CapabilityBasisBuilder {
    fn new(subject: &str) -> Self {
        Self {
            subject: subject.to_string(),
            entries: Vec::with_capacity(80),
        }
    }

    fn text(&mut self, locus: impl Into<String>, value: impl AsRef<str>) {
        self.value(
            locus,
            CanonicalBasisValue::ExactText(value.as_ref().to_owned().into()),
        );
    }

    fn digest(&mut self, locus: impl Into<String>, value: &CanonicalDigestId) {
        self.value(locus, CanonicalBasisValue::BytesDigest(*value));
    }

    fn value(&mut self, locus: impl Into<String>, value: CanonicalBasisValue) {
        self.entries.push(entry(locus, value));
    }

    fn finish(
        self,
    ) -> Result<
        WorthQueryCapabilityCanonicalArtifact,
        WorthQueryApplicationCapabilityInstallationDenial,
    > {
        let version = CanonicalizationRuleVersion::new(RULE_VERSION)
            .expect("the installed capability rule is valid");
        let basis = prepare_canonical_basis_sequence(version, DOMAIN, self.entries)
            .into_result()
            .expect("installed capability basis loci are unique and typed");
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(
                basis.clone(),
                CanonicalDigestAlgorithmId::sha256(),
                CAPABILITY_BUDGET,
            )
            .into_result()
            .map_err(|denial| canonical_denial(&self.subject, denial))?;
        let derived = canonicalization().digest().derive(ready);
        Ok(WorthQueryCapabilityCanonicalArtifact {
            basis,
            digest: CanonicalDigestId::new(*derived.value().bytes()),
            work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
        })
    }
}

fn entry(locus: impl Into<String>, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into().into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}

fn canonical_denial(
    subject: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryApplicationCapabilityInstallationDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalEntryLimitExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalByteLimitExceeded
        }
        _ => WorthQueryApplicationCapabilityInstallationDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryApplicationCapabilityInstallationDenial::new(kind, subject)
}
