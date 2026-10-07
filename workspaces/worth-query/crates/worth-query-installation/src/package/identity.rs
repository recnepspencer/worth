use worth_foundational::facade::{
    CanonicalDigestDerivationDenial, CanonicalDigestId, CanonicalDigestWorkBudget,
};

use super::WorthQueryPortableDomainPackage;
use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::canonical_work::{
    WorthQueryCanonicalWorkEvidence, INSTALLATION_MAXIMUM_CANONICAL_BYTES,
};

mod schema_commitments;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryPortableDomainIdentity {
    owner: String,
    major: u32,
    minor: u32,
}

impl WorthQueryPortableDomainIdentity {
    pub fn new(owner: impl Into<String>, major: u32, minor: u32) -> Self {
        Self {
            owner: owner.into(),
            major,
            minor,
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn major(&self) -> u32 {
        self.major
    }

    pub fn minor(&self) -> u32 {
        self.minor
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthQueryPortableDomainPackageIdentity(CanonicalDigestId);

impl WorthQueryPortableDomainPackageIdentity {
    /// Reconstruct descriptive claimed identity from untrusted boundary bytes.
    ///
    /// This value carries no validation or installation authority. Phase 3
    /// compares it with identity freshly derived from reconstructed meaning.
    pub const fn from_untrusted_bytes(bytes: [u8; 32]) -> Self {
        Self(CanonicalDigestId::new(bytes))
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.0
    }

    pub const fn bytes(&self) -> &[u8; 32] {
        self.0.bytes()
    }

    pub fn render_support_hex(&self) -> String {
        self.0.render_hex()
    }
}

pub(super) fn canonical_identity_with_maximum_bytes(
    package: &WorthQueryPortableDomainPackage,
    maximum_canonical_bytes: usize,
) -> Result<
    (
        WorthQueryPortableDomainPackageIdentity,
        WorthQueryCanonicalWorkEvidence,
    ),
    CanonicalDigestDerivationDenial,
> {
    let budget = preflight_canonical_entries(package, maximum_canonical_bytes)?;
    let schemas = schema_commitments::admit(&package.application_schemas, budget)?;
    let mut basis = InstallationCanonicalIdentityBasis::new(
        "worth-query.portable-domain-package",
        "worth-query-portable-domain-package-v4",
        schemas.remaining_budget()?,
    );
    append_domain_identity(&mut basis, package)?;
    append_requirements(&mut basis, package)?;
    append_definitions(&mut basis, package)?;
    append_domain_operations(&mut basis, package)?;
    append_contracts_and_schemas(&mut basis, package, schemas.digests())?;
    append_conditional_application_operations(&mut basis, package)?;
    for (index, contribution) in package.contributions.iter().enumerate() {
        basis.text(format!("contribution[{index}]"), contribution.as_str())?;
    }
    let (digest, work) = basis
        .derive()
        .map_err(|denial| schemas.aggregate_denial(denial))?;
    Ok((
        WorthQueryPortableDomainPackageIdentity(digest),
        schemas.work().combine(work),
    ))
}

/// Reject impossible entry breadth before member sorting or basis allocation.
pub(super) fn preflight_canonical_entries(
    package: &WorthQueryPortableDomainPackage,
    maximum_canonical_bytes: usize,
) -> Result<CanonicalDigestWorkBudget, CanonicalDigestDerivationDenial> {
    // Storage admission follows the installation's finite ceiling. A caller's
    // narrower byte policy continues to report exact encoded-byte exhaustion
    // rather than being reinterpreted as an independent cardinality policy.
    let storage =
        CanonicalDigestWorkBudget::for_encoded_byte_ceiling(INSTALLATION_MAXIMUM_CANONICAL_BYTES)
            .expect("fixed installation byte ceiling is nonzero");
    let budget = CanonicalDigestWorkBudget::new(
        storage.maximum_entry_count(),
        maximum_canonical_bytes.min(INSTALLATION_MAXIMUM_CANONICAL_BYTES),
    )
    .ok_or(CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
        maximum: 0,
        attempted: 0,
    })?;
    let mut entries = 3usize;
    for count in [
        package.capabilities.len(),
        package.configuration.len(),
        package.operating.len(),
        package.definitions.len().saturating_mul(3),
        package.domain_operations.len().saturating_mul(2),
        package.artifact_contracts.len(),
        package
            .conditional_application_operations
            .len()
            .saturating_mul(6),
        package.contributions.len(),
    ] {
        entries = entries.saturating_add(count);
    }
    for schema in &package.application_schemas {
        entries = entries.saturating_add(3).saturating_add(
            schema
                .identity()
                .canonical_basis()
                .payload()
                .entries()
                .len(),
        );
    }
    let actual = u32::try_from(entries).unwrap_or(u32::MAX);
    if actual > budget.maximum_entry_count() {
        return Err(CanonicalDigestDerivationDenial::EntryLimitExceeded {
            maximum: budget.maximum_entry_count(),
            actual,
        });
    }
    Ok(budget)
}

fn append_conditional_application_operations(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), CanonicalDigestDerivationDenial> {
    for (index, binding) in package
        .conditional_application_operations
        .iter()
        .enumerate()
    {
        let prefix = format!("conditional-application-operation[{index}]");
        basis.text(format!("{prefix}.schema-owner"), binding.schema_owner())?;
        basis.text(format!("{prefix}.schema-name"), binding.schema_name())?;
        basis.text(
            format!("{prefix}.application-operation"),
            binding.application_operation(),
        )?;
        basis.text(format!("{prefix}.input-type"), binding.input_type())?;
        basis.text(
            format!("{prefix}.domain-operation-slot"),
            binding.domain_operation_slot(),
        )?;
        basis.text(
            format!("{prefix}.domain-operation-identity"),
            binding.domain_operation_canonical_identity(),
        )?;
    }
    Ok(())
}

fn append_domain_identity(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), CanonicalDigestDerivationDenial> {
    basis.text("domain.owner", package.identity.owner())?;
    basis.unsigned_u32("domain.major", package.identity.major())?;
    basis.unsigned_u32("domain.minor", package.identity.minor())
}

fn append_requirements(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), CanonicalDigestDerivationDenial> {
    for (index, capability) in package.capabilities.iter().enumerate() {
        basis.text(format!("capability[{index}]"), capability.as_str())?;
    }
    for (index, configuration) in package.configuration.iter().enumerate() {
        basis.text(format!("configuration[{index}]"), configuration.as_str())?;
    }
    for (index, operating) in package.operating.iter().enumerate() {
        basis.text(format!("operating[{index}]"), operating.as_str())?;
    }
    Ok(())
}

fn append_definitions(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), CanonicalDigestDerivationDenial> {
    for (index, definition) in package.definitions.iter().enumerate() {
        let prefix = format!("definition[{index}]");
        basis.text(format!("{prefix}.kind"), definition.kind().as_str())?;
        basis.text(format!("{prefix}.slot"), definition.slot())?;
        basis.text(format!("{prefix}.semantics"), definition.semantics())?;
    }
    Ok(())
}

fn append_domain_operations(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
) -> Result<(), CanonicalDigestDerivationDenial> {
    for (index, operation) in package.domain_operations.iter().enumerate() {
        let prefix = format!("domain-operation[{index}]");
        basis.text(format!("{prefix}.slot"), operation.identity().slot())?;
        basis.text(format!("{prefix}.identity"), operation.canonical_identity())?;
    }
    Ok(())
}

fn append_contracts_and_schemas(
    basis: &mut InstallationCanonicalIdentityBasis,
    package: &WorthQueryPortableDomainPackage,
    schema_digests: &[CanonicalDigestId],
) -> Result<(), CanonicalDigestDerivationDenial> {
    for (index, contract) in package.artifact_contracts.iter().enumerate() {
        basis.text(
            format!("artifact-contract[{index}].identity"),
            contract.identity().as_str(),
        )?;
    }
    for (index, (schema, digest)) in package
        .application_schemas
        .iter()
        .zip(schema_digests)
        .enumerate()
    {
        let prefix = format!("application-schema[{index}]");
        basis.text(format!("{prefix}.owner"), schema.owner())?;
        basis.text(format!("{prefix}.name"), schema.name())?;
        basis.digest(format!("{prefix}.meaning"), *digest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod schema_commitment_tests;
