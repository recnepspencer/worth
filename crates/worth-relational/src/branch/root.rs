use std::sync::Arc;

use super::target::RelationalBranchRootDescriptor;
use super::RelationalPersistentRegionSet;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::RelationalPublishedPartitionDelta;

#[path = "root_allocation.rs"]
mod allocation;
#[cfg(test)]
#[path = "root_authoritative_payload_tests.rs"]
mod authoritative_payload_tests;
#[path = "root_axes.rs"]
mod axes;
#[path = "root_capture.rs"]
mod capture;
#[path = "root_capture_preparation.rs"]
mod capture_preparation;
#[path = "root_commit_evidence.rs"]
mod commit_evidence;
#[path = "root_empty.rs"]
mod empty;
pub(crate) use commit_evidence::RelationalRootCommitEvidence;
#[cfg(test)]
#[path = "root_content_tests.rs"]
mod content_tests;
#[path = "root_identity.rs"]
mod identity;
#[path = "root_output_publication.rs"]
mod output_publication;
#[path = "root_owner_allocation_ledger.rs"]
mod owner_allocation_ledger;
#[cfg(test)]
#[path = "root_physical_linkage_tests.rs"]
mod physical_linkage_tests;
#[path = "root_readmission.rs"]
mod readmission;
#[path = "root_region_access.rs"]
mod region_access;
#[path = "root_schema.rs"]
mod schema;
#[path = "root_schema_meaning.rs"]
mod schema_meaning;
#[cfg(test)]
#[path = "root_tests.rs"]
mod tests;
#[path = "root_visibility.rs"]
mod visibility;

pub(super) use super::root_region::RelationalRootRegion;
pub(crate) use super::root_region::RelationalRootRegionObservation;
pub(super) use allocation::{reserve_arc_memory, reserve_box_memory};
pub(crate) use allocation::{
    RelationalRootAllocationVisitor, RelationalRootAuthoritativeAllocationKind,
    RelationalRootAuthoritativeAllocationObservation,
};
pub(crate) use identity::RelationalBranchRootIdentityIssuer;
pub(crate) use schema::{
    retained_compilation_tree_bytes, RelationalBranchRootSchemaAuthority,
    RetainedRelationalAspectPlans, RetainedRelationalSchemaRegistry,
};
pub(crate) use visibility::RelationalBranchVisibilityCommitment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelationalBranchRootCaptureDenial {
    PublishedPartitionMissing(PartitionId),
    DuplicateRegion(PartitionId),
    InvalidAdjacencyProjection(PartitionId),
    InvalidHistoryCoverage(PartitionId),
    ContentEncoding(PartitionId),
    AllocationAccountingOverflow(PartitionId),
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
    SlotCountOverflow,
    AllocationLayoutOverflow,
    Request(crate::execution::RelationalExecutionDenialCause),
    CanonicalEnvelopeMismatch,
    StorageContentMismatch {
        descriptor_root: [u8; 32],
        reconstructed_root: [u8; 32],
    },
    SchemaRootMismatch {
        descriptor_root: [u8; 32],
        canonical_root: [u8; 32],
    },
    VisibilityCommitmentMismatch {
        committed: [u8; 32],
        reconstructed: [u8; 32],
    },
    UnresolvedContentSymbol(crate::symbols::data::Symbol),
    RootIdentityExhausted,
    SchemaAuthorityIdentityExhausted,
    RegionIdentityExhausted,
    ReachabilityIdentityExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct RelationalBranchRootPublicationCost {
    pub(crate) touched_regions: u64,
    pub(crate) reused_regions: u64,
    /// Exact immutable radix nodes allocated by this publication.
    pub(crate) persistent_index_path_nodes: u64,
    pub(crate) content_values_hashed: u64,
    pub(crate) new_authoritative_bytes: u64,
    pub(crate) copied_truth_bytes: u64,
    pub(crate) copied_commit_envelopes: u64,
    pub(crate) new_schema_authorities: u64,
    pub(crate) reused_schema_authorities: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRootCorrectnessIndex {
    AuthoritativeFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RelationalBranchRootAxes {
    pub(crate) storage_version: u64,
    /// Inspection/completeness digest, never a currentness selector.
    pub(crate) storage_root: [u8; 32],
    pub(crate) schema_root: [u8; 32],
    pub(crate) correctness_index: RelationalRootCorrectnessIndex,
    pub(crate) visibility: RelationalBranchVisibilityCommitment,
}

#[derive(Debug, Clone)]
struct RelationalCommittedBranchRoot {
    descriptor: RelationalBranchRootDescriptor,
    evidence: RelationalRootCommitEvidence,
    axes: RelationalBranchRootAxes,
    publication_cost: RelationalBranchRootPublicationCost,
}

/// Immutable root selected by one branch reference.
///
/// A later root replaces only regions named by the storage owner's sealed
/// publication delta. Untouched region `Arc`s are retained exactly; no hash,
/// debug representation, or collision-prone equality proxy can authorize
/// reuse.
#[derive(Debug)]
pub(crate) struct RelationalBranchRoot {
    id: u64,
    regions: Arc<RelationalPersistentRegionSet>,
    entity_slot_count: usize,
    relation_slot_count: usize,
    content_accumulator: [u8; 32],
    pub(crate) output_publication_positions:
        crate::history::data::RequiredOutputPublicationPositions,
    schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub(crate) execution_contracts:
        std::sync::OnceLock<Arc<crate::runtime::RelationalRootExecutionContracts>>,
    committed: Option<RelationalCommittedBranchRoot>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub(crate) managed_source_image: std::sync::OnceLock<crate::mvcc::ManagedSourceRootImage>,
}

#[path = "root_managed_fork.rs"]
mod managed_fork;

#[derive(Debug, Clone)]
pub(crate) struct RelationalBranchRootState {
    pub(super) root: Arc<RelationalBranchRoot>,
}

pub(crate) struct PreparedRelationalBranchRootCapture {
    root: Arc<RelationalBranchRoot>,
}

impl PreparedRelationalBranchRootCapture {
    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }

    pub(crate) fn into_root(self) -> Arc<RelationalBranchRoot> {
        self.root
    }
}

impl RelationalBranchRootState {
    pub(crate) fn new(root: Arc<RelationalBranchRoot>) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
}

impl RelationalBranchRoot {
    pub(crate) fn prepare_capture(
        issuer: &RelationalBranchRootIdentityIssuer,
        partitions: &impl PartitionAccess,
        published_delta: &RelationalPublishedPartitionDelta,
        previous: Option<&Arc<Self>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::runtime::RuntimeSymbolTable,
        execution_contracts: Option<Arc<crate::runtime::RelationalRootExecutionContracts>>,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        capture_preparation::prepare(
            issuer,
            partitions,
            published_delta,
            previous,
            envelope,
            registry,
            symbols,
            execution_contracts,
        )
    }
    pub(crate) const fn id(&self) -> u64 {
        self.id
    }

    pub(crate) const fn entity_slot_count(&self) -> usize {
        self.entity_slot_count
    }

    pub(crate) const fn relation_slot_count(&self) -> usize {
        self.relation_slot_count
    }

    pub(crate) fn descriptor(&self) -> Option<&RelationalBranchRootDescriptor> {
        self.committed.as_ref().map(|root| &root.descriptor)
    }

    pub(crate) fn axes(&self) -> Option<RelationalBranchRootAxes> {
        self.committed.as_ref().map(|root| root.axes)
    }

    pub(crate) fn is_complete(&self, symbols: &crate::symbols::data::StringInterner) -> bool {
        let Some(committed) = self.committed.as_ref() else {
            return false;
        };
        let Ok((reconstructed_root, reconstructed_accumulator)) =
            axes::storage_root_from_authoritative_regions(&self.regions, symbols)
        else {
            return false;
        };
        let canonical_schema_root = self.schema_authority.authority_digest();
        let reconstructed_visibility = match &committed.evidence {
            RelationalRootCommitEvidence::Original(envelope) => {
                RelationalBranchVisibilityCommitment::for_root(
                    envelope,
                    reconstructed_root,
                    canonical_schema_root,
                    committed.axes.correctness_index,
                )
            }
            RelationalRootCommitEvidence::Checkpoint(commit) => commit.visibility(),
        };
        committed.axes.storage_root == reconstructed_root
            && self.content_accumulator == reconstructed_accumulator
            && committed.axes.storage_root == *committed.descriptor.truth_root()
            && committed.axes.schema_root == *committed.descriptor.schema_root()
            && committed.axes.schema_root == canonical_schema_root
            && committed.evidence.matches_schema(&self.schema_authority)
            && committed.axes.storage_version == committed.evidence.receipt().version_id.0
            && committed.axes.storage_root != [0; 32]
            && committed.axes.visibility == reconstructed_visibility
    }

    pub(crate) fn commit_id(&self) -> Option<CommitId> {
        self.committed
            .as_ref()
            .map(|root| root.evidence.receipt().commit_id)
    }

    pub(crate) fn canonical_envelope(&self) -> Option<&Arc<CanonicalCommitEnvelope>> {
        self.commit_evidence()
            .and_then(RelationalRootCommitEvidence::original)
    }

    pub(crate) fn commit_evidence(&self) -> Option<&RelationalRootCommitEvidence> {
        self.committed.as_ref().map(|root| &root.evidence)
    }

    pub(crate) fn commit_receipt(&self) -> Option<&crate::history::data::RelationalCommitReceipt> {
        self.commit_evidence()
            .map(RelationalRootCommitEvidence::receipt)
    }

    pub(crate) fn schema_authority(&self) -> &RelationalBranchRootSchemaAuthority {
        &self.schema_authority
    }

    pub(crate) fn retained_schema_authority(&self) -> Arc<RelationalBranchRootSchemaAuthority> {
        Arc::clone(&self.schema_authority)
    }

    pub(crate) fn storage_regions(
        &self,
    ) -> impl Iterator<Item = RelationalRootRegionObservation> + '_ {
        self.regions.values().map(|region| region.observation())
    }

    pub(crate) fn region_count(&self) -> usize {
        self.regions.len()
    }

    pub(crate) fn has_materialization_unavailable(&self) -> bool {
        self.regions.materialization_unavailable_records() > 0
    }

    /// Logical authoritative partition payload bytes reachable from this root.
    pub(crate) fn logical_partition_payload_bytes(&self) -> u64 {
        self.regions
            .values()
            .map(|region| region.allocation_inventory().authoritative_bytes)
            .sum()
    }

    pub(crate) fn publication_cost(&self) -> RelationalBranchRootPublicationCost {
        self.committed
            .as_ref()
            .map(|root| root.publication_cost)
            .unwrap_or_default()
    }

    pub(crate) fn reclaimable_unique_authoritative_bytes(&self) -> u64 {
        let mut bytes = std::mem::size_of::<Self>() as u64;
        bytes = bytes.saturating_add(
            RelationalPersistentRegionSet::reclaimable_unique_authoritative_bytes(&self.regions),
        );
        if Arc::strong_count(&self.schema_authority) == 1 {
            bytes = bytes.saturating_add(self.schema_authority.authoritative_allocation_bytes());
        }
        bytes
    }

    pub(crate) fn links_envelope(&self, envelope: &Arc<CanonicalCommitEnvelope>) -> bool {
        self.committed.as_ref().is_some_and(|root| {
            root.evidence
                .original()
                .is_some_and(|original| Arc::ptr_eq(original, envelope))
                && root.axes.storage_root == *root.descriptor.truth_root()
                && root.axes.schema_root == *root.descriptor.schema_root()
                && root.axes.visibility
                    == RelationalBranchVisibilityCommitment::for_root(
                        envelope,
                        root.axes.storage_root,
                        root.axes.schema_root,
                        root.axes.correctness_index,
                    )
        })
    }

    pub(crate) fn shares_canonical_envelope_with(&self, source: &Self) -> bool {
        match (&self.committed, &source.committed) {
            (Some(target), Some(source)) => target.evidence.shares_owner(&source.evidence),
            (None, None) => true,
            _ => false,
        }
    }
}
