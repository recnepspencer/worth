//! Current typed record membership, maintained from the sealed commit delta.
mod maintenance;
mod routing;
pub(crate) use routing::KindRegionDirectory;
mod membership;
mod portable;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
pub(crate) use maintenance::DirectoryError;
use membership::Membership;
pub(crate) use portable::PortableRecordDirectory;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub(crate) struct RegionRecordDirectory {
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
    pub(crate) fn partition_id(&self) -> PartitionId {
        self.partition
    }
    pub(crate) fn empty(partition: PartitionId) -> Self {
        Self {
            partition,
            entities: Membership::default(),
            relations: Membership::default(),
            portable: OnceLock::new(),
        }
    }
    pub(crate) fn digest(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        digest.update(b"worth.region.record-directory.v1\0");
        digest.update(self.partition.0.to_be_bytes());
        digest.update(self.entities.digest());
        digest.update(self.relations.digest());
        digest.finalize().into()
    }
    pub(crate) fn entity_ids(&self, kind: KindId) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.ids(kind, self.partition)
    }
    pub(crate) fn relation_ids(&self, kind: KindId) -> impl Iterator<Item = RelationId> + '_ {
        self.relations.ids(kind, self.partition)
    }
    pub(crate) fn changed_entity_kinds(
        &self,
        previous: &Self,
    ) -> impl Iterator<Item = KindId> + use<> {
        self.entities.changed_kinds(&previous.entities).into_iter()
    }
    pub(crate) fn changed_relation_kinds(
        &self,
        previous: &Self,
    ) -> impl Iterator<Item = KindId> + use<> {
        self.relations
            .changed_kinds(&previous.relations)
            .into_iter()
    }
}
