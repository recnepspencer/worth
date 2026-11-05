use std::collections::BTreeMap;
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
#[path = "root_completeness.rs"]
mod completeness;
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
pub(crate) use schema::RelationalBranchRootSchemaAuthority;
pub(crate) use visibility::RelationalBranchVisibilityCommitment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RelationalBranchRootCaptureDenial {
    Execution(crate::execution::RelationalExecutionDenialCause),
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
    UnadmittedContent,
    Content(crate::content::RelationalContentDenial),
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
    content: crate::content::ContentDirectory,
    id: u64,
    regions: Arc<RelationalPersistentRegionSet>,
    entity_slot_count: usize,
    relation_slot_count: usize,
    content_accumulator: [u8; 32],
    pub(crate) output_publication_positions:
        crate::history::data::RequiredOutputPublicationPositions,
    schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
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
    #[cfg(test)]
    pub(crate) fn empty() -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        Arc::new(Self {
            id: 0,
            regions: RelationalPersistentRegionSet::initial(0, BTreeMap::new(), &issuer)
                .expect("empty test root has reachability capacity"),
            content: Default::default(),
            entity_slot_count: 0,
            relation_slot_count: 0,
            content_accumulator: [0; 32],
            output_publication_positions: Default::default(),
            schema_authority: RelationalBranchRootSchemaAuthority::empty(),
            committed: None,
            _memory: None,
            managed_source_image: std::sync::OnceLock::new(),
        })
    }

    pub(crate) fn empty_with_schema(
        registry: &crate::schema::data::RelationalSchemaRegistry,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
    ) -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        let expected = registry.authority_snapshot();
        let schema_authority_id = issuer
            .issue_schema_authority_id()
            .expect(/* Proof: Each empty root creates a fresh issuer at its initial frontier; these first schema and empty-reachability allocations cannot exhaust it. */ "empty root has schema authority capacity");
        let schema = RelationalBranchRootSchemaAuthority::capture(
            schema_authority_id, registry, &expected, descriptor_semantics_version, None,
        ).expect(/* Proof: the authority is captured from this same validated registry and its own snapshot. */ "empty schema matches registry");
        Self::empty_with_schema_authority(schema)
    }

    pub(crate) fn empty_with_schema_authority(
        schema: Arc<RelationalBranchRootSchemaAuthority>,
    ) -> Arc<Self> {
        let issuer = RelationalBranchRootIdentityIssuer::default();
        Arc::new(Self {
            id: 0,
            regions: RelationalPersistentRegionSet::initial(0, BTreeMap::new(), &issuer)
                .expect(/* Proof: Each empty root creates a fresh issuer at its initial frontier; these first schema and empty-reachability allocations cannot exhaust it. */ "empty root has reachability capacity"),
            entity_slot_count: 0,
            relation_slot_count: 0,
            content: Default::default(),
            content_accumulator: [0; 32],
            output_publication_positions: Default::default(),
            schema_authority: schema,
            committed: None,
            _memory: None,
            managed_source_image: std::sync::OnceLock::new(),
        })
    }
    pub(crate) fn prepare_capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        issuer: &RelationalBranchRootIdentityIssuer,
        partitions: &impl PartitionAccess,
        published_delta: &RelationalPublishedPartitionDelta,
        previous: Option<&Arc<Self>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        content: &crate::content::ContentPublication,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        capture_preparation::prepare(
            execution,
            issuer,
            partitions,
            published_delta,
            previous,
            envelope,
            content,
            registry,
            symbols,
        )
    }
    pub(crate) fn retained_content(&self) -> &crate::content::ContentDirectory {
        &self.content
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
