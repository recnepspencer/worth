use super::staging_storage::OrderedStore;
use super::RelationalTransactionStagingDenial as Denial;
use std::collections::BTreeSet;

use crate::identity::data::{KindId, PartitionId};
use crate::transactions::data::{CreatedEntityRef, CreatedRelationRef, RecordRef};

mod client_keys;
mod validation_dependencies;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationalTransactionReadLocus {
    Existing(RecordRef),
    CreatedEntity(CreatedEntityRef),
    CreatedRelation(CreatedRelationRef),
    ValidationPartition(PartitionId),
    EntitySchema(KindId),
    RelationSchema(KindId),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RelationalTransactionWriteLocus {
    Existing(RecordRef),
    CreatedEntity(CreatedEntityRef),
    CreatedRelation(CreatedRelationRef),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalTransactionFootprint {
    pub(super) basis: crate::branch::RelationalBranchBasisDescriptor,
    pub(super) reads: OrderedStore<RelationalTransactionReadLocus>,
    pub(super) writes: OrderedStore<RelationalTransactionWriteLocus>,
    pub(super) write_partitions: OrderedStore<PartitionId>,
}

impl RelationalTransactionFootprint {
    pub(crate) fn for_basis(basis: &crate::branch::AdmittedRelationalBranchBasis) -> Self {
        Self {
            basis: basis.descriptor().clone(),
            reads: OrderedStore::default(),
            writes: OrderedStore::default(),
            write_partitions: OrderedStore::default(),
        }
    }

    pub fn basis(&self) -> &crate::branch::RelationalBranchBasisDescriptor {
        &self.basis
    }

    pub fn branch(&self) -> &crate::history::data::BranchId {
        self.basis.branch_id()
    }

    pub fn reference(&self) -> &crate::branch::RelationalBranchReferenceObservation {
        self.basis.reference()
    }

    pub fn reads(&self) -> impl ExactSizeIterator<Item = &RelationalTransactionReadLocus> {
        self.reads.iter()
    }

    pub fn writes(&self) -> impl ExactSizeIterator<Item = &RelationalTransactionWriteLocus> {
        self.writes.iter()
    }

    pub fn write_partitions(&self) -> impl ExactSizeIterator<Item = &PartitionId> {
        self.write_partitions.iter()
    }

    pub(crate) fn record_read(
        &mut self,
        locus: RelationalTransactionReadLocus,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), Denial> {
        self.reads = self
            .reads
            .with_inserted(std::iter::once(locus), allocation_policy)?;
        Ok(())
    }

    pub(super) fn total_locus_count(&self) -> usize {
        self.reads.len().saturating_add(self.writes.len())
    }

    pub(crate) fn validation_partitions(&self) -> BTreeSet<PartitionId> {
        let mut partitions = self
            .write_partitions
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for read in &self.reads {
            match read {
                RelationalTransactionReadLocus::Existing(RecordRef::Entity(entity)) => {
                    partitions.insert(entity.partition_id);
                }
                RelationalTransactionReadLocus::Existing(RecordRef::Relation(relation)) => {
                    partitions.insert(relation.partition_id);
                }
                RelationalTransactionReadLocus::CreatedEntity(entity) => {
                    partitions.insert(entity.partition_id);
                }
                RelationalTransactionReadLocus::CreatedRelation(relation) => {
                    partitions.insert(relation.partition_id);
                }
                RelationalTransactionReadLocus::ValidationPartition(partition) => {
                    partitions.insert(*partition);
                }
                RelationalTransactionReadLocus::EntitySchema(_)
                | RelationalTransactionReadLocus::RelationSchema(_) => {}
            }
        }
        partitions
    }
}
