use serde::{Deserialize, Serialize};

mod retained_charge;

use crate::data::handle::NodeId;
use crate::data::node::ContextRequirement;
use crate::data::output::PartitionSubscription;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArtifactFamilyId(String);

impl ArtifactFamilyId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReuseStrategyBoundaryContext {
    #[default]
    None,
    CrossIdentity {
        persistent_correspondence: PersistentCorrespondenceEvidence,
    },
    PartialArtifactSplice {
        composition_regions: crate::data::proof::PartitionScopeSet,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PersistentCorrespondenceKind {
    Unknown,
    HostSuppliedKey,
    ContractDeclaredBasis,
    LineageBackedMapping,
    RegionIdentityBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistentCorrespondenceEvidence {
    HostSuppliedKey(String),
    ContractDeclaredBasis(String),
    LineageBackedMapping(String),
    RegionIdentityBasis(String),
}

impl PersistentCorrespondenceEvidence {
    pub fn host_supplied_key(value: impl Into<String>) -> Self {
        Self::HostSuppliedKey(value.into())
    }

    pub fn contract_declared_basis(value: impl Into<String>) -> Self {
        Self::ContractDeclaredBasis(value.into())
    }

    pub fn lineage_backed_mapping(value: impl Into<String>) -> Self {
        Self::LineageBackedMapping(value.into())
    }

    pub fn region_identity_basis(value: impl Into<String>) -> Self {
        Self::RegionIdentityBasis(value.into())
    }

    pub fn kind(&self) -> PersistentCorrespondenceKind {
        match self {
            Self::HostSuppliedKey(_) => PersistentCorrespondenceKind::HostSuppliedKey,
            Self::ContractDeclaredBasis(_) => PersistentCorrespondenceKind::ContractDeclaredBasis,
            Self::LineageBackedMapping(_) => PersistentCorrespondenceKind::LineageBackedMapping,
            Self::RegionIdentityBasis(_) => PersistentCorrespondenceKind::RegionIdentityBasis,
        }
    }

    pub fn is_structurally_valid(&self) -> bool {
        match self {
            Self::HostSuppliedKey(value) => !value.trim().is_empty(),
            Self::ContractDeclaredBasis(value) => {
                let trimmed = value.trim();
                trimmed.starts_with("contract:")
                    && trimmed.len() > "contract:".len()
                    && !trimmed.contains('|')
            }
            Self::LineageBackedMapping(value) => {
                let trimmed = value.trim();
                if !trimmed.starts_with("lineage-map:") || trimmed.contains('|') {
                    return false;
                }
                let mapping = &trimmed["lineage-map:".len()..];
                let mut segments = mapping.split("->");
                let Some(left) = segments.next() else {
                    return false;
                };
                let Some(right) = segments.next() else {
                    return false;
                };
                segments.next().is_none() && !left.trim().is_empty() && !right.trim().is_empty()
            }
            Self::RegionIdentityBasis(value) => {
                let trimmed = value.trim();
                trimmed.starts_with("region:")
                    && trimmed.len() > "region:".len()
                    && !trimmed.contains('|')
            }
        }
    }
}

/// Stable node-local semantic region identity for one artifact family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReuseSemanticRegionIdentity {
    pub node: NodeId,
    pub partitioned_output: bool,
    #[serde(default)]
    pub partition_scope: Vec<PartitionSubscription>,
    #[serde(default)]
    pub required_context: ContextRequirement,
}

impl ReuseSemanticRegionIdentity {
    pub fn new(
        node: NodeId,
        partitioned_output: bool,
        partition_scope: impl Into<Vec<PartitionSubscription>>,
        required_context: ContextRequirement,
    ) -> Self {
        let mut partition_scope = partition_scope.into();
        if partition_scope.len() > 1 {
            partition_scope.sort_unstable();
            partition_scope.dedup();
        }
        Self {
            node,
            partitioned_output,
            partition_scope,
            required_context,
        }
    }
}
