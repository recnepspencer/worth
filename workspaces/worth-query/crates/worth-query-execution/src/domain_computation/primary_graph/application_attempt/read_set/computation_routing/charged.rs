//! What the sealed facts of a partitioned computation hold beyond their
//! inline values: every key's names and locator, every fact's owned values
//! and lists, and every reader list, as `ChargedBytes` declares them.

use worth_execution::ChargedBytes;
use worth_proof::CanonicalUniqueVec;

use super::super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use super::{ComputationFactReaders, SealedComputationFact, SealedComputationFacts};

fn bytes(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}

/// A list's allocation: its capacity of inline values. The listed values own
/// nothing more.
fn list<T>(values: &Vec<T>) -> u64 {
    bytes(values.capacity().saturating_mul(std::mem::size_of::<T>()))
}

/// A checked list's allocation: its capacity of inline keys.
fn canonical<T>(values: &CanonicalUniqueVec<T>) -> u64 {
    bytes(values.capacity().saturating_mul(std::mem::size_of::<T>()))
}

impl ChargedBytes for WorthQueryApplicationFactKey {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Entity { entity, .. } => bytes(entity.capacity()),
            Self::Field {
                entity, locator, ..
            } => bytes(entity.capacity())
                .saturating_add(bytes(locator.owned_allocation_capacity_bytes())),
            Self::Relation { relation, .. } | Self::Adjacency { relation, .. } => {
                bytes(relation.capacity())
            }
        }
    }
}

impl ChargedBytes for WorthQueryApplicationObservedFact {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::RetiredOutputEntity { read_locator, .. } => bytes(read_locator.capacity()),
            Self::SourceAspectRevision { aspect, .. } => {
                bytes(aspect.owned_allocation_capacity_bytes())
            }
            Self::SourceFieldRevision { locator, .. } | Self::AbsentField { locator, .. } => {
                bytes(locator.owned_allocation_capacity_bytes())
            }
            Self::SourceAdjacencyRevision { endpoints, .. } => list(endpoints),
            Self::Field { locator, value, .. } => bytes(locator.owned_allocation_capacity_bytes())
                .saturating_add(bytes(value.owned_allocation_capacity_bytes())),
            Self::Relation {
                matching_relations, ..
            } => list(matching_relations),
            Self::Adjacency { relations, .. } => list(relations),
            // The index definition is shared with the index, not owned here.
            Self::IndexedEntitySelection {
                locator,
                value,
                candidates,
                ..
            } => bytes(locator.owned_allocation_capacity_bytes())
                .saturating_add(bytes(value.owned_allocation_capacity_bytes()))
                .saturating_add(list(candidates)),
            Self::WorkflowInstanceCapacity { instances, .. } => list(instances),
            Self::SourceEntity { .. }
            | Self::Entity { .. }
            | Self::WorkflowDefinitionPredecessor { .. }
            | Self::WorkflowDefinitionCurrent { .. }
            | Self::WorkflowHistoryBasis { .. } => 0,
        }
    }
}

impl ChargedBytes for ComputationFactReaders {
    fn additional_charged_bytes(&self) -> u64 {
        canonical(&self.item_keys).saturating_add(canonical(&self.partitions))
    }
}

impl ChargedBytes for SealedComputationFact {
    fn additional_charged_bytes(&self) -> u64 {
        self.fact
            .additional_charged_bytes()
            .saturating_add(self.readers.additional_charged_bytes())
    }
}

impl SealedComputationFacts {
    /// What retaining the facts holds: every entry's key, fact and readers
    /// inline, and what each owns beyond that. `None` when the sum overflows.
    pub(in crate::domain_computation::primary_graph) fn charged_bytes(&self) -> Option<u64> {
        let entry = u64::try_from(
            std::mem::size_of::<WorthQueryApplicationFactKey>()
                + std::mem::size_of::<SealedComputationFact>(),
        )
        .ok()?;
        self.facts.iter().try_fold(0_u64, |sum, (key, sealed)| {
            sum.checked_add(entry)?
                .checked_add(key.additional_charged_bytes())?
                .checked_add(sealed.additional_charged_bytes())
        })
    }
}
