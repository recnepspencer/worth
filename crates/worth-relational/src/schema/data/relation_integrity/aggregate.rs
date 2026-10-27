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

/// The installed graph whose connected groups receive persistent names.
/// The scalar orbit encoding is retained exactly. Components have a distinct
/// object grammar and read every declared relation as undirected coupling.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(untagged)]
pub enum AggregateIdentitySource {
    Orbit(KindId),
    Components { relations: Vec<KindId> },
}

impl Serialize for AggregateIdentitySource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        match self {
            Self::Orbit(relation) => relation.serialize(serializer),
            Self::Components { relations } => {
                // Compact MessagePack encodes derived structs as sequences.
                // Preserve an object here so every format has one decoder.
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("relations", relations)?;
                map.end()
            }
        }
    }
}

impl AggregateIdentitySource {
    pub fn orbit_relation(&self) -> Option<KindId> {
        match self {
            Self::Orbit(relation) => Some(*relation),
            Self::Components { .. } => None,
        }
    }

    pub fn relations(&self) -> &[KindId] {
        match self {
            Self::Orbit(relation) => std::slice::from_ref(relation),
            Self::Components { relations } => relations,
        }
    }

    /// Runtime contracts have already normalized the component roster.
    pub(crate) fn contains_relation(&self, kind: KindId) -> bool {
        match self {
            Self::Orbit(relation) => *relation == kind,
            Self::Components { relations } => relations.binary_search(&kind).is_ok(),
        }
    }

    pub(crate) fn canonicalize(&mut self) {
        if let Self::Components { relations } = self {
            relations.sort();
            relations.dedup();
        }
    }

    pub(crate) fn owned_bytes(&self) -> u64 {
        match self {
            Self::Orbit(_) => 0,
            Self::Components { relations } => {
                (relations.capacity() * std::mem::size_of::<KindId>()) as u64
            }
        }
    }
}

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
    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(68);
        bytes.extend_from_slice(self.definition.bytes());
        match &self.source {
            AggregateIdentitySource::Orbit(relation) => {
                bytes.extend_from_slice(&relation.0.to_be_bytes())
            }
            AggregateIdentitySource::Components { relations } => {
                bytes.extend_from_slice(b"WORTH_COMPONENT_SOURCES_V1\0");
                bytes.extend_from_slice(&(relations.len() as u64).to_be_bytes());
                for relation in relations {
                    bytes.extend_from_slice(&relation.0.to_be_bytes());
                }
            }
        }
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
        bytes
    }
}
