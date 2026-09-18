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
const RULE_VERSION: &str = "worth-query-installed-index-v3";
// Across all installed packages, this identity retains compact installed names
// and slot-level semantics. It does not embed application-schema meaning, so it
// has its own smaller bound.
const INDEX_BUDGET: CanonicalDigestWorkBudget =
    match CanonicalDigestWorkBudget::new(32_768, 4 * 1_024 * 1_024) {
        Some(budget) => budget,
        None => panic!("fixed installed-index canonical-work budget is valid"),
    };

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
    append_definitions(&mut entries, input.definitions);
    append_domain_operations(&mut entries, input.domain_operations);
    append_artifact_contracts(&mut entries, input.artifact_contracts);
    append_application_schemas(&mut entries, input.application_schemas);
    append_conditional_application_operations(
        &mut entries,
        input.conditional_application_operations,
    );

    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the installed-index identity rule is valid");
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .expect("installed-index identity loci are unique and typed");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), INDEX_BUDGET)
        .into_result()?;
    let derived = canonicalization().digest().derive(ready);
    Ok((
        WorthQueryInstalledPackageIndexIdentity(CanonicalDigestId::new(*derived.value().bytes())),
        WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    ))
}

fn append_conditional_application_operations(
    entries: &mut Vec<CanonicalBasisEntry>,
    bindings: &BTreeMap<
        (String, String, String),
        WorthQueryPortableApplicationConditionalOperationBinding,
    >,
) {
    for (index, ((owner, schema, application_operation), binding)) in bindings.iter().enumerate() {
        let prefix = format!("conditional-application-operation[{index}]");
        entries.extend([
            text(format!("{prefix}.owner"), owner),
            text(format!("{prefix}.schema"), schema),
            text(
                format!("{prefix}.application-operation"),
                application_operation,
            ),
            text(format!("{prefix}.input-type"), binding.input_type()),
            text(
                format!("{prefix}.domain-operation-slot"),
                binding.domain_operation_slot(),
            ),
            text(
                format!("{prefix}.domain-operation-identity"),
                binding.domain_operation_canonical_identity(),
            ),
        ]);
    }
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

fn append_definitions(
    entries: &mut Vec<CanonicalBasisEntry>,
    definitions: &BTreeMap<
        (WorthQueryPortableDefinitionKind, String, String),
        WorthQueryPortableDefinition,
    >,
) {
    for (index, ((kind, owner, slot), definition)) in definitions.iter().enumerate() {
        let prefix = format!("definition[{index}]");
        entries.extend([
            text(format!("{prefix}.kind"), kind.as_str()),
            text(format!("{prefix}.owner"), owner),
            text(format!("{prefix}.slot"), slot),
            text(format!("{prefix}.semantics"), definition.semantics()),
        ]);
    }
}

fn append_domain_operations(
    entries: &mut Vec<CanonicalBasisEntry>,
    operations: &BTreeMap<(String, String), WorthQueryValidatedDomainOperation>,
) {
    for (index, ((owner, slot), operation)) in operations.iter().enumerate() {
        let prefix = format!("domain-operation[{index}]");
        entries.extend([
            text(format!("{prefix}.owner"), owner),
            text(format!("{prefix}.slot"), slot),
            text(
                format!("{prefix}.identity"),
                operation.definition().canonical_identity(),
            ),
        ]);
    }
}

fn append_artifact_contracts(
    entries: &mut Vec<CanonicalBasisEntry>,
    contracts: &BTreeMap<(String, String, u32, u32), WorthQueryPortableArtifactContract>,
) {
    for (index, ((owner, family, schema, protocol), contract)) in contracts.iter().enumerate() {
        let prefix = format!("artifact-contract[{index}]");
        entries.extend([
            text(format!("{prefix}.owner"), owner),
            text(format!("{prefix}.family"), family),
            unsigned32(format!("{prefix}.schema"), *schema),
            unsigned32(format!("{prefix}.protocol"), *protocol),
            text(format!("{prefix}.identity"), contract.identity().as_str()),
        ]);
    }
}

fn append_application_schemas(
    entries: &mut Vec<CanonicalBasisEntry>,
    schemas: &BTreeMap<(String, String), WorthQueryInstalledApplicationSchemaRecord>,
) {
    for (index, ((owner, name), record)) in schemas.iter().enumerate() {
        let schema = record.declaration();
        let prefix = format!("application-schema[{index}]");
        entries.extend([
            text(format!("{prefix}.owner"), owner),
            text(format!("{prefix}.name"), name),
            text(format!("{prefix}.declared-owner"), schema.owner()),
        ]);
    }
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

fn unsigned32(locus: impl Into<String>, value: u32) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits32,
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
