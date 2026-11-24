//! Current typed record membership, maintained from the sealed commit delta.
pub mod maintenance;
pub mod routing;
pub use routing::KindRegionDirectory;
pub mod membership;
pub mod portable;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
pub use maintenance::DirectoryError;
use membership::Membership;
pub use portable::PortableRecordDirectory;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub struct RegionRecordDirectory {
    partition: PartitionId,
    entities: Membership,
    relations: Membership,
    portable: OnceLock<Arc<PortableRecordDirectory>>,
}
impl Clone for RegionRecordDirectory {
    fn clone(&self) -> Self {
        Self {
            partition: self.partition,
            entities: self.entities.clone(),
            relations: self.relations.clone(),
            portable: OnceLock::new(),
        }
    }
}
impl RegionRecordDirectory {
    pub fn partition_id(&self) -> PartitionId {
        self.partition
    }
    pub fn empty(partition: PartitionId) -> Self {
        Self {
            partition,
            entities: Membership::default(),
            relations: Membership::default(),
            portable: OnceLock::new(),
        }
    }
    pub fn digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth.region.record-directory.v1\0");
        digest.update(self.partition.0.to_be_bytes());
        digest.update(self.entities.digest());
        digest.update(self.relations.digest());
        digest.finalize().into()
    }
    pub fn entity_ids(&self, kind: KindId) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.ids(kind, self.partition)
    }
    pub fn relation_ids(&self, kind: KindId) -> impl Iterator<Item = RelationId> + '_ {
        self.relations.ids(kind, self.partition)
    }
    pub fn changed_entity_kinds(&self, previous: &Self) -> impl Iterator<Item = KindId> + use<> {
        self.entities.changed_kinds(&previous.entities).into_iter()
    }
    pub fn changed_relation_kinds(&self, previous: &Self) -> impl Iterator<Item = KindId> + use<> {
        self.relations
            .changed_kinds(&previous.relations)
            .into_iter()
    }
}
