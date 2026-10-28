//! Explicit aggregate ownership, distinct from structural cardinality alone.
use serde::{Deserialize, Serialize};
use worth_foundational::facade::{AggregateDefinitionIdentity, AspectIdentity};

use crate::identity::data::KindId;

/// Reserved platform storage shared by aggregate definitions in one installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AggregateIdentityStorage {
    pub aggregate_kind: KindId,
    pub lineage_kind: KindId,
    pub aggregate_aspect: AspectIdentity,
    pub lineage_aspect: AspectIdentity,
    /// Absent keeps the original four-field storage shape and record grammar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projected: Option<AggregateProjectedMemberStorage>,
}

/// Reserved grammar for projected coordinates and grouped carrier memberships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AggregateProjectedMemberStorage {
    pub coordinate_aspect: AspectIdentity,
    pub membership_aspect: AspectIdentity,
}

/// The installed physical or registered derived graph receiving persistent names.
mod source;
pub use source::AggregateIdentitySource;

/// Installed ownership of a stored graph's aggregate records.
/// This is schema meaning, not a handle granting a caller metadata write access.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AggregateIdentityContractDeclaration {
    pub definition: AggregateDefinitionIdentity,
    #[serde(rename = "source_relation")]
    pub source: AggregateIdentitySource,
    pub member_kind: KindId,
    pub membership_kind: KindId,
    pub storage: AggregateIdentityStorage,
}

impl AggregateIdentityContractDeclaration {
    /// These kinds remain Native custody across authored payload suspension.
    pub(crate) fn metadata_kinds(&self) -> [KindId; 3] {
        [
            self.storage.aggregate_kind,
            self.storage.lineage_kind,
            self.membership_kind,
        ]
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(68);
        bytes.extend_from_slice(self.definition.bytes());
        self.source.write_canonical(&mut bytes);
        for kind in [
            self.member_kind,
            self.membership_kind,
            self.storage.aggregate_kind,
            self.storage.lineage_kind,
        ] {
            bytes.extend_from_slice(&kind.0.to_be_bytes());
        }
        bytes.extend_from_slice(&self.storage.aggregate_aspect.0.to_be_bytes());
        bytes.extend_from_slice(&self.storage.lineage_aspect.0.to_be_bytes());
        if let Some(projected) = self.storage.projected {
            bytes.extend_from_slice(b"WORTH_PROJECTED_AGGREGATE_STORAGE_V1\0");
            bytes.extend_from_slice(&projected.coordinate_aspect.0.to_be_bytes());
            bytes.extend_from_slice(&projected.membership_aspect.0.to_be_bytes());
        }
        bytes
    }
}
