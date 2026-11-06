use std::collections::BTreeMap;

use super::{
    application_schema_record::WorthQueryInstalledApplicationSchemaRecord,
    WorthQueryInstalledPackageRecord,
};
use crate::application_operation::WorthQueryPortableApplicationConditionalOperationBinding;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence;
use crate::domain_computation::WorthQueryPortableArtifactContract;
use crate::domain_operation::WorthQueryValidatedDomainOperation;
use crate::generation::{WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity};
use crate::package::{WorthQueryPortableDefinition, WorthQueryPortableDefinitionKind};
use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalIntegerWidth, CanonicalizationRuleVersion,
};

const DOMAIN: CanonicalBasisDomain = CanonicalBasisDomain::Future("worth-query.installed-index");
const RULE_VERSION: &str = "worth-query-installed-index-v4";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledPackageIndexIdentity(CanonicalDigestId);

impl WorthQueryInstalledPackageIndexIdentity {
    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }

    #[cfg(test)]
    pub(super) fn corrupt_for_test(&mut self) {
        let mut bytes = *self.0.bytes();
        bytes[0] ^= 0xff;
        self.0 = CanonicalDigestId::new(bytes);
    }
}

pub(super) struct IndexIdentityInput<'a> {
    pub runtime: &'a WorthQueryInstallationRuntimeIdentity,
    pub generation: WorthQueryInstallationGeneration,
    pub records: &'a BTreeMap<String, WorthQueryInstalledPackageRecord>,
    pub definitions: &'a BTreeMap<
        (WorthQueryPortableDefinitionKind, String, String),
        WorthQueryPortableDefinition,
    >,
    pub domain_operations: &'a BTreeMap<(String, String), WorthQueryValidatedDomainOperation>,
    pub artifact_contracts:
        &'a BTreeMap<(String, String, u32, u32), WorthQueryPortableArtifactContract>,
    pub application_schemas:
        &'a BTreeMap<(String, String), WorthQueryInstalledApplicationSchemaRecord>,
    pub conditional_application_operations: &'a BTreeMap<
        (String, String, String),
        WorthQueryPortableApplicationConditionalOperationBinding,
    >,
}

pub(super) fn index_identity(
    input: IndexIdentityInput<'_>,
) -> Result<
    (
        WorthQueryInstalledPackageIndexIdentity,
        WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let mut entries = vec![
        unsigned("runtime", input.runtime.ordinal()),
        unsigned("generation", input.generation.ordinal()),
        count("package-count", input.records.len()),
        count("definition-count", input.definitions.len()),
        count("domain-operation-count", input.domain_operations.len()),
        count("artifact-contract-count", input.artifact_contracts.len()),
        count("application-schema-count", input.application_schemas.len()),
        count(
            "conditional-application-operation-count",
            input.conditional_application_operations.len(),
        ),
    ];
    append_records(&mut entries, input.records);
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the installed-index identity rule is valid");
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .expect("installed-index identity loci are unique and typed");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(
            basis,
            CanonicalDigestAlgorithmId::sha256(),
            index_budget(input.records),
        )
        .into_result()?;
    let derived = canonicalization().digest().derive(ready);
    Ok((
        WorthQueryInstalledPackageIndexIdentity(CanonicalDigestId::new(*derived.value().bytes())),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}

fn append_records(
    entries: &mut Vec<CanonicalBasisEntry>,
    records: &BTreeMap<String, WorthQueryInstalledPackageRecord>,
) {
    for (index, (owner, record)) in records.iter().enumerate() {
        let prefix = format!("package[{index}]");
        entries.extend([
            text(format!("{prefix}.owner"), owner),
            digest(
                format!("{prefix}.identity"),
                record.package.package().identity().digest(),
            ),
            digest(
                format!("{prefix}.admission-identity"),
                record.package.admission_identity().digest(),
            ),
        ]);
    }
}

// Catalogs are deterministic projections of the validated packages. Their
// identities already commit every definition, schema and operation exactly.
// Re-embedding semantics here both duplicates truth and can amplify long owners
// once per definition. The v4 index commits the source package/admission table.
fn index_budget(
    records: &BTreeMap<String, WorthQueryInstalledPackageRecord>,
) -> CanonicalDigestWorkBudget {
    let (entries, bytes) = records
        .values()
        .fold((0u32, 0usize), |(entries, bytes), record| {
            let budget = crate::derived_identity_budget::derived_identity_budget(
                record.package.package().canonical_work(),
            );
            (
                entries.saturating_add(budget.maximum_entry_count()),
                bytes.saturating_add(budget.maximum_encoded_bytes()),
            )
        });
    // An empty installed index has only its eight fixed framing entries.
    CanonicalDigestWorkBudget::new(entries.max(8), bytes.max(4096))
        .expect("index framing is nonempty")
}

fn text(locus: impl Into<String>, value: &str) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn digest(
    locus: impl Into<String>,
    value: &worth_foundational::facade::CanonicalDigestId,
) -> CanonicalBasisEntry {
    entry(locus, CanonicalBasisValue::BytesDigest(*value))
}

fn unsigned(locus: impl Into<String>, value: u64) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: value.into(),
        },
    )
}

fn count(locus: impl Into<String>, value: usize) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: value as u128,
        },
    )
}

fn entry(locus: impl Into<String>, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into().into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}
