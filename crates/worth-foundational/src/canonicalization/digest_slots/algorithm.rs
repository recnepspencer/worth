use std::marker::PhantomData;

use super::super::{CanonicalBasisDomain, CanonicalizationRuleVersion};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalDigestAlgorithmId(String);

impl CanonicalDigestAlgorithmId {
    // V1 included preparation work in identity. It is deliberately unsupported:
    // persisted v1 identities must be re-derived from their canonical basis at
    // upgrade, never relabeled or compared as v2 identities.
    pub const SHA256_POLICY_ID: &'static str = "sha256-canonical-v2";

    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() || value.contains(char::is_whitespace) {
            None
        } else {
            Some(Self(value))
        }
    }

    pub fn sha256() -> Self {
        Self(Self::SHA256_POLICY_ID.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn is_supported(&self) -> bool {
        self.is_sha256()
    }

    pub(super) fn is_sha256(&self) -> bool {
        self.0 == Self::SHA256_POLICY_ID
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestInputShape {
    SingleSequence,
    DomainBundle,
    ExportBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestOutputShape {
    Bytes32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalDigestInputDomain {
    Single(CanonicalBasisDomain),
    DomainBundle,
    ExportBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalSingleSequenceDigestInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalDomainBundleDigestInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalExportBundleDigestInput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDigestAlgorithmSlot<S> {
    metadata: CanonicalDigestAlgorithmMetadata,
    shape: PhantomData<S>,
}

pub type CanonicalSingleSequenceDigestAlgorithmSlot =
    CanonicalDigestAlgorithmSlot<CanonicalSingleSequenceDigestInput>;
pub type CanonicalDomainBundleDigestAlgorithmSlot =
    CanonicalDigestAlgorithmSlot<CanonicalDomainBundleDigestInput>;
pub type CanonicalExportBundleDigestAlgorithmSlot =
    CanonicalDigestAlgorithmSlot<CanonicalExportBundleDigestInput>;

impl CanonicalSingleSequenceDigestAlgorithmSlot {
    pub fn single_sequence(
        id: CanonicalDigestAlgorithmId,
        domain: CanonicalBasisDomain,
        rule_version: CanonicalizationRuleVersion,
    ) -> Self {
        Self::new(CanonicalDigestAlgorithmMetadata::new(
            id,
            CanonicalDigestInputDomain::Single(domain),
            rule_version,
            CanonicalDigestInputShape::SingleSequence,
            CanonicalDigestOutputShape::Bytes32,
        ))
    }
}

impl CanonicalDomainBundleDigestAlgorithmSlot {
    pub fn domain_bundle(
        id: CanonicalDigestAlgorithmId,
        rule_version: CanonicalizationRuleVersion,
    ) -> Self {
        Self::new(CanonicalDigestAlgorithmMetadata::new(
            id,
            CanonicalDigestInputDomain::DomainBundle,
            rule_version,
            CanonicalDigestInputShape::DomainBundle,
            CanonicalDigestOutputShape::Bytes32,
        ))
    }
}

impl CanonicalExportBundleDigestAlgorithmSlot {
    pub fn export_bundle(
        id: CanonicalDigestAlgorithmId,
        rule_version: CanonicalizationRuleVersion,
    ) -> Self {
        Self::new(CanonicalDigestAlgorithmMetadata::new(
            id,
            CanonicalDigestInputDomain::ExportBundle,
            rule_version,
            CanonicalDigestInputShape::ExportBundle,
            CanonicalDigestOutputShape::Bytes32,
        ))
    }
}

impl<S> CanonicalDigestAlgorithmSlot<S> {
    fn new(metadata: CanonicalDigestAlgorithmMetadata) -> Self {
        Self {
            metadata,
            shape: PhantomData,
        }
    }

    pub fn metadata(&self) -> &CanonicalDigestAlgorithmMetadata {
        &self.metadata
    }

    pub(super) fn into_metadata(self) -> CanonicalDigestAlgorithmMetadata {
        self.metadata
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalDigestAlgorithmMetadata {
    id: CanonicalDigestAlgorithmId,
    input_domain: CanonicalDigestInputDomain,
    rule_version: CanonicalizationRuleVersion,
    input_shape: CanonicalDigestInputShape,
    output_shape: CanonicalDigestOutputShape,
}

impl CanonicalDigestAlgorithmMetadata {
    fn new(
        id: CanonicalDigestAlgorithmId,
        input_domain: CanonicalDigestInputDomain,
        rule_version: CanonicalizationRuleVersion,
        input_shape: CanonicalDigestInputShape,
        output_shape: CanonicalDigestOutputShape,
    ) -> Self {
        Self {
            id,
            input_domain,
            rule_version,
            input_shape,
            output_shape,
        }
    }

    pub fn id(&self) -> &CanonicalDigestAlgorithmId {
        &self.id
    }

    pub const fn input_domain(&self) -> CanonicalDigestInputDomain {
        self.input_domain
    }

    pub fn rule_version(&self) -> &CanonicalizationRuleVersion {
        &self.rule_version
    }

    pub const fn input_shape(&self) -> CanonicalDigestInputShape {
        self.input_shape
    }

    pub const fn output_shape(&self) -> CanonicalDigestOutputShape {
        self.output_shape
    }
}
