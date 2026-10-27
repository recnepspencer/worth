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
}

/// Installed ownership of a stored homogeneous permutation's aggregate records.
/// This is schema meaning, not a handle granting a caller metadata write access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AggregateIdentityContractDeclaration {
    pub definition: AggregateDefinitionIdentity,
    pub source_relation: KindId,
    pub member_kind: KindId,
    pub membership_kind: KindId,
    pub storage: AggregateIdentityStorage,
}

impl AggregateIdentityContractDeclaration {
    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(68);
        bytes.extend_from_slice(self.definition.bytes());
        for kind in [
            self.source_relation,
            self.member_kind,
            self.membership_kind,
            self.storage.aggregate_kind,
            self.storage.lineage_kind,
        ] {
            bytes.extend_from_slice(&kind.0.to_be_bytes());
        }
        bytes.extend_from_slice(&self.storage.aggregate_aspect.0.to_be_bytes());
        bytes.extend_from_slice(&self.storage.lineage_aspect.0.to_be_bytes());
        bytes
    }
}
