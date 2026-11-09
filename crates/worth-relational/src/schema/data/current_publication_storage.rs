//! Physical grammar of the Native-owned current publication roster.
use crate::identity::data::KindId;
use serde::{Deserialize, Serialize};
use worth_foundational::facade::AspectIdentity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPublicationStorage {
    pub catalog_kind: KindId,
    pub publication_kind: KindId,
    pub edge_kind: KindId,
    pub position_kind: KindId,
    pub position_link: KindId,
    pub publication_link: KindId,
    pub member_link: KindId,
    pub edge_link: KindId,
    pub scope_link: KindId,
    pub catalog_aspect: AspectIdentity,
    pub publication_aspect: AspectIdentity,
    pub edge_aspect: AspectIdentity,
    pub position_aspect: AspectIdentity,
    pub link_aspect: AspectIdentity,
    pub entity_kinds: Vec<KindId>,
}

impl CurrentPublicationStorage {
    pub(crate) fn metadata_kinds(&self) -> [KindId; 9] {
        [
            self.catalog_kind,
            self.publication_kind,
            self.edge_kind,
            self.position_kind,
            self.position_link,
            self.publication_link,
            self.member_link,
            self.edge_link,
            self.scope_link,
        ]
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = b"WORTH_CURRENT_PUBLICATION_STORAGE_V2\0".to_vec();
        for kind in self.metadata_kinds() {
            bytes.extend(kind.0.to_be_bytes());
        }
        for aspect in [
            self.catalog_aspect,
            self.publication_aspect,
            self.edge_aspect,
            self.position_aspect,
            self.link_aspect,
        ] {
            bytes.extend(aspect.0.to_be_bytes());
        }
        bytes.extend((self.entity_kinds.len() as u64).to_be_bytes());
        for kind in &self.entity_kinds {
            bytes.extend(kind.0.to_be_bytes());
        }
        bytes
    }
}
