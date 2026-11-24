use super::staging_storage::OrderedStore;
use crate::contracts::mvcc::transaction::staging::RelationalTransactionStagingDenial as Denial;
use std::collections::BTreeSet;

use crate::contracts::identity::data::primitives::{KindId, PartitionId};
use crate::contracts::transactions::data::primitives::{
    CreatedEntityRef, CreatedRelationRef, RecordRef,
};

pub mod client_keys;
pub mod retained_selection;
pub mod validation_dependencies;

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
    pub(super) basis: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    pub(super) reads: OrderedStore<RelationalTransactionReadLocus>,
    pub(super) writes: OrderedStore<RelationalTransactionWriteLocus>,
    pub(super) write_partitions: OrderedStore<PartitionId>,
}

impl RelationalTransactionFootprint {
    pub(crate) fn for_basis(
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            basis: basis.descriptor().clone(),
            reads: OrderedStore::default(),
            writes: OrderedStore::default(),
            write_partitions: OrderedStore::default(),
        }
    }

    pub fn basis(&self) -> &worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor{
        &self.basis
    }

    pub fn branch(&self) -> &crate::contracts::history::data::coordinates::BranchId {
        self.basis.branch_id()
    }

    pub fn reference(
        &self,
    ) -> &crate::contracts::branch::reference::observation::RelationalBranchReferenceObservation
    {
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
            .with_inserted(std::iter::once(Ok(locus)), allocation_policy)?;
        Ok(())
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

#[doc(hidden)]
pub trait RelationalTransactionFootprintOwnerPort: Sized {
    fn for_basis(
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Self;

    fn validation_partitions(&self) -> BTreeSet<PartitionId>;
}

impl RelationalTransactionFootprintOwnerPort for RelationalTransactionFootprint {
    fn for_basis(
        basis: &worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    ) -> Self {
        Self::for_basis(basis)
    }

    fn validation_partitions(&self) -> BTreeSet<PartitionId> {
        self.validation_partitions()
    }
}
