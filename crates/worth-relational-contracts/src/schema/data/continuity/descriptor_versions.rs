use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DescriptorSemanticsVersion(pub u32);

impl Default for DescriptorSemanticsVersion {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptorSemanticsSupportPolicy {
    current_write_version: DescriptorSemanticsVersion,
    supported_historical_versions: BTreeSet<DescriptorSemanticsVersion>,
}

impl DescriptorSemanticsSupportPolicy {
    pub fn new(
        current_write_version: DescriptorSemanticsVersion,
        supported_historical_versions: impl IntoIterator<Item = DescriptorSemanticsVersion>,
    ) -> Self {
        let mut supported_historical_versions = supported_historical_versions
            .into_iter()
            .collect::<BTreeSet<_>>();
        supported_historical_versions.insert(current_write_version);
        Self {
            current_write_version,
            supported_historical_versions,
        }
    }

    pub fn current_write_version(&self) -> DescriptorSemanticsVersion {
        self.current_write_version
    }

    pub fn supports(&self, version: DescriptorSemanticsVersion) -> bool {
        self.supported_historical_versions.contains(&version)
    }
}

impl Default for DescriptorSemanticsSupportPolicy {
    fn default() -> Self {
        Self::new(
            DescriptorSemanticsVersion::default(),
            [DescriptorSemanticsVersion::default()],
        )
    }
}

pub fn runtime_descriptor_semantics_policy() -> DescriptorSemanticsSupportPolicy {
    DescriptorSemanticsSupportPolicy::default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DescriptorCanonicalBasisVersion(pub u32);

impl Default for DescriptorCanonicalBasisVersion {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptorCanonicalBasisSupportPolicy {
    current_write_version: DescriptorCanonicalBasisVersion,
    supported_historical_versions: BTreeSet<DescriptorCanonicalBasisVersion>,
}

impl DescriptorCanonicalBasisSupportPolicy {
    pub fn new(
        current_write_version: DescriptorCanonicalBasisVersion,
        supported_historical_versions: impl IntoIterator<Item = DescriptorCanonicalBasisVersion>,
    ) -> Self {
        let mut supported_historical_versions = supported_historical_versions
            .into_iter()
            .collect::<BTreeSet<_>>();
        supported_historical_versions.insert(current_write_version);
        Self {
            current_write_version,
            supported_historical_versions,
        }
    }

    pub fn current_write_version(&self) -> DescriptorCanonicalBasisVersion {
        self.current_write_version
    }

    pub fn supports(&self, version: DescriptorCanonicalBasisVersion) -> bool {
        self.supported_historical_versions.contains(&version)
    }
}

impl Default for DescriptorCanonicalBasisSupportPolicy {
    fn default() -> Self {
        Self::new(
            DescriptorCanonicalBasisVersion::default(),
            [DescriptorCanonicalBasisVersion::default()],
        )
    }
}

pub fn runtime_descriptor_canonical_basis_policy() -> DescriptorCanonicalBasisSupportPolicy {
    DescriptorCanonicalBasisSupportPolicy::default()
}
