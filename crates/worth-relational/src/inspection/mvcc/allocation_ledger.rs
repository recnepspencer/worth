use std::collections::BTreeSet;

use crate::branch::{RelationalBranchIdentity, RelationalRootAuthoritativeAllocationKind};
use crate::history::data::CommitId;
use crate::history::{RelationalCommitArtifact, RelationalCommitAuthoritativeAllocationKind};
use crate::identity::data::PartitionId;
use crate::runtime::RelationalRuntime;

use super::sharing::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation, RelationalBranchSharingInspectionDenial,
};

/// Owner inventory used to cross-examine the derived sharing summary.
///
/// Entries come directly from root, partition, and canonical-artifact owners;
/// this artifact does not call or reuse `observe_branch_sharing`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalOwnerAllocationLedgerObservation {
    authoritative_allocations: Vec<RelationalAuthoritativeAllocationObservation>,
    excluded_allocations: Vec<RelationalOwnerExcludedAllocationObservation>,
    canonical_payloads: Vec<RelationalCanonicalPayloadObservation>,
}

impl RelationalOwnerAllocationLedgerObservation {
    pub fn authoritative_allocations(&self) -> &[RelationalAuthoritativeAllocationObservation] {
        &self.authoritative_allocations
    }

    pub fn excluded_allocations(&self) -> &[RelationalOwnerExcludedAllocationObservation] {
        &self.excluded_allocations
    }

    pub fn canonical_payloads(&self) -> &[RelationalCanonicalPayloadObservation] {
        &self.canonical_payloads
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalCanonicalPayloadObservation {
    commit_id: CommitId,
    digest: [u8; 32],
}

impl RelationalCanonicalPayloadObservation {
    pub const fn commit_id(self) -> CommitId {
        self.commit_id
    }

    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationalExcludedAllocationLane {
    Diagnostics,
    RetentionMetadata,
    AllocatorBookkeeping,
    OptionalCache,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalOwnerExcludedAllocationObservation {
    runtime_instance_id: u64,
    lane: RelationalExcludedAllocationLane,
    owner_id: u64,
    partition_id: Option<PartitionId>,
    bytes: u64,
}

impl RelationalOwnerExcludedAllocationObservation {
    pub const fn lane(self) -> RelationalExcludedAllocationLane {
        self.lane
    }

    pub const fn owner_id(self) -> u64 {
        self.owner_id
    }

    pub const fn partition_id(self) -> Option<PartitionId> {
        self.partition_id
    }

    pub const fn bytes(self) -> u64 {
        self.bytes
    }
}

impl RelationalRuntime {
    pub fn inspect_owner_allocation_ledger(
        &self,
        branches: &[RelationalBranchIdentity],
    ) -> Result<RelationalOwnerAllocationLedgerObservation, RelationalBranchSharingInspectionDenial>
    {
        let mut authoritative_allocations = Vec::new();
        let mut excluded_allocations = Vec::new();
        let mut canonical_payloads = BTreeSet::new();
        let mut seen_branches = BTreeSet::new();
        for identity in branches {
            if identity.runtime_instance_id() != self.runtime_instance_id() {
                return Err(RelationalBranchSharingInspectionDenial::ForeignRuntime);
            }
            if !seen_branches.insert(identity.clone()) {
                return Err(RelationalBranchSharingInspectionDenial::DuplicateBranch);
            }
            let cell = self
                .history
                .branch_cell(identity.branch_id())
                .filter(|cell| cell.identity() == identity)
                .ok_or(RelationalBranchSharingInspectionDenial::UnknownBranch)?;
            let root = cell
                .root()
                .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
            let commit_id = root
                .commit_id()
                .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
            let artifact = self
                .history
                .commit_artifact(commit_id)
                .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
            canonical_payloads.insert(RelationalCanonicalPayloadObservation {
                commit_id,
                digest: artifact.canonical_payload_digest(),
            });
            let derived_cache_bytes = self
                .indexes
                .derived_artifacts_for_commit(commit_id)
                .owned_allocation_capacity_bytes();
            inventory_root(
                self.runtime_instance_id(),
                &root,
                artifact.as_ref(),
                derived_cache_bytes,
                &mut authoritative_allocations,
                &mut excluded_allocations,
            );
        }
        Ok(RelationalOwnerAllocationLedgerObservation {
            authoritative_allocations,
            excluded_allocations,
            canonical_payloads: canonical_payloads.into_iter().collect(),
        })
    }
}

fn inventory_root(
    runtime_instance_id: u64,
    root: &crate::branch::RelationalBranchRoot,
    artifact: &RelationalCommitArtifact,
    derived_cache_bytes: u64,
    authoritative: &mut Vec<RelationalAuthoritativeAllocationObservation>,
    excluded: &mut Vec<RelationalOwnerExcludedAllocationObservation>,
) {
    for region in root.storage_regions() {
        let partition = root
            .partition_state(region.partition_id)
            .expect("observed region owns its partition");
        let mut payload = PayloadLedger {
            runtime_instance_id,
            partition_id: region.partition_id,
            seen: Default::default(),
            entries: authoritative,
        };
        partition.visit_authoritative_allocations(false, &mut payload);
        for (kind, bytes) in [
            (
                RelationalAuthoritativeAllocationKind::PartitionStateObject,
                region.partition_state_bytes,
            ),
            (
                RelationalAuthoritativeAllocationKind::RootRegionObject,
                region.root_region_bytes,
            ),
        ] {
            authoritative.push(RelationalAuthoritativeAllocationObservation::new(
                RelationalAuthoritativeAllocationLocator::new(
                    runtime_instance_id,
                    kind,
                    region.region_id,
                    region.creation_root_id,
                    Some(region.partition_id),
                ),
                bytes,
            ));
        }
        let mut payload = ExcludedPayloadLedger {
            runtime_instance_id,
            partition_id: region.partition_id,
            lane: RelationalExcludedAllocationLane::Diagnostics,
            seen: Default::default(),
            entries: excluded,
        };
        partition.visit_diagnostic_allocations(&mut payload);
        payload.lane = RelationalExcludedAllocationLane::RetentionMetadata;
        partition.visit_retention_allocations(&mut payload);
        payload.lane = RelationalExcludedAllocationLane::OptionalCache;
        partition.visit_cache_allocations(&mut payload);
        root.visit_content_cache_allocations(region.partition_id, &mut payload);
    }
    for allocation in root.owner_allocation_ledger_entries() {
        let kind = root_allocation_kind(allocation.kind);
        authoritative.push(RelationalAuthoritativeAllocationObservation::new(
            RelationalAuthoritativeAllocationLocator::new(
                runtime_instance_id,
                kind,
                allocation.owner_id,
                allocation.owner_id,
                None,
            ),
            allocation.authoritative_bytes,
        ));
    }
    inventory_commit(
        runtime_instance_id,
        artifact,
        derived_cache_bytes,
        authoritative,
        excluded,
    );
}

struct PayloadLedger<'a> {
    runtime_instance_id: u64,
    partition_id: PartitionId,
    seen: BTreeSet<u64>,
    entries: &'a mut Vec<RelationalAuthoritativeAllocationObservation>,
}

impl crate::storage::substrate::StorageAllocationVisitor for PayloadLedger<'_> {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if !self.seen.insert(allocation.id) {
            return false;
        }
        self.entries
            .push(RelationalAuthoritativeAllocationObservation::new(
                RelationalAuthoritativeAllocationLocator::new(
                    self.runtime_instance_id,
                    RelationalAuthoritativeAllocationKind::PartitionPayload,
                    allocation.id,
                    allocation.id,
                    Some(self.partition_id),
                ),
                allocation.bytes,
            ));
        true
    }
}

fn inventory_commit(
    runtime_instance_id: u64,
    artifact: &RelationalCommitArtifact,
    derived_cache_bytes: u64,
    authoritative: &mut Vec<RelationalAuthoritativeAllocationObservation>,
    excluded: &mut Vec<RelationalOwnerExcludedAllocationObservation>,
) {
    for allocation in artifact.authoritative_allocation_observations() {
        authoritative.push(RelationalAuthoritativeAllocationObservation::new(
            RelationalAuthoritativeAllocationLocator::new(
                runtime_instance_id,
                commit_allocation_kind(allocation.kind),
                artifact.commit_id().0,
                artifact.commit_id().0,
                None,
            ),
            allocation.authoritative_bytes,
        ));
    }
    let inventory = artifact.excluded_allocation_inventory();
    excluded.extend([
        RelationalOwnerExcludedAllocationObservation {
            runtime_instance_id,
            lane: RelationalExcludedAllocationLane::Diagnostics,
            owner_id: artifact.commit_id().0,
            partition_id: None,
            bytes: inventory.diagnostic_bytes,
        },
        RelationalOwnerExcludedAllocationObservation {
            runtime_instance_id,
            lane: RelationalExcludedAllocationLane::OptionalCache,
            owner_id: artifact.commit_id().0,
            partition_id: None,
            bytes: inventory
                .optional_cache_bytes
                .saturating_add(derived_cache_bytes),
        },
    ]);
}

struct ExcludedPayloadLedger<'a> {
    runtime_instance_id: u64,
    partition_id: PartitionId,
    lane: RelationalExcludedAllocationLane,
    seen: BTreeSet<u64>,
    entries: &'a mut Vec<RelationalOwnerExcludedAllocationObservation>,
}

impl crate::storage::substrate::StorageAllocationVisitor for ExcludedPayloadLedger<'_> {
    fn visit(
        &mut self,
        allocation: crate::storage::substrate::StorageAllocationObservation,
    ) -> bool {
        if !self.seen.insert(allocation.id) {
            return false;
        }
        self.entries
            .push(RelationalOwnerExcludedAllocationObservation {
                runtime_instance_id: self.runtime_instance_id,
                lane: self.lane,
                owner_id: allocation.id,
                partition_id: Some(self.partition_id),
                bytes: allocation.bytes,
            });
        true
    }
}

fn root_allocation_kind(
    kind: RelationalRootAuthoritativeAllocationKind,
) -> RelationalAuthoritativeAllocationKind {
    match kind {
        RelationalRootAuthoritativeAllocationKind::RootMetadata => {
            RelationalAuthoritativeAllocationKind::RootMetadata
        }
        RelationalRootAuthoritativeAllocationKind::SchemaAuthority => {
            RelationalAuthoritativeAllocationKind::RootSchemaAuthority
        }
        RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject => {
            RelationalAuthoritativeAllocationKind::RootReachabilitySetObject
        }
        RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject => {
            RelationalAuthoritativeAllocationKind::RootReachabilityStructure
        }
        RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage => {
            RelationalAuthoritativeAllocationKind::RootReplacementStorage
        }
        RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage => {
            RelationalAuthoritativeAllocationKind::RootRemovalStorage
        }
    }
}

fn commit_allocation_kind(
    kind: RelationalCommitAuthoritativeAllocationKind,
) -> RelationalAuthoritativeAllocationKind {
    match kind {
        RelationalCommitAuthoritativeAllocationKind::ArtifactObject => {
            RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact
        }
        RelationalCommitAuthoritativeAllocationKind::CanonicalPayload => {
            RelationalAuthoritativeAllocationKind::CanonicalCommitPayload
        }
        RelationalCommitAuthoritativeAllocationKind::EnvelopeObject => {
            RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope
        }
        RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage => {
            RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested
        }
    }
}
