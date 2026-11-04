//! The eager directory of the current native checkpoint. Record payloads,
//! history, and rebuildable projections belong to members, never this directory.
use serde::{Deserialize, Serialize};

use super::*;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug, Clone)]
pub(crate) struct RelationalCheckpointManifest(std::sync::Arc<ManifestBacking>);
#[derive(Debug)]
struct ManifestBacking {
    data: RelationalCheckpointManifestDraft,
    _control: Option<worth_execution::ExecutionMemoryReservation>,
}
impl std::ops::Deref for RelationalCheckpointManifest {
    type Target = RelationalCheckpointManifestDraft;
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}
impl PartialEq for RelationalCheckpointManifest {
    fn eq(&self, other: &Self) -> bool {
        self.0.data == other.0.data
    }
}
impl Eq for RelationalCheckpointManifest {}
impl RelationalCheckpointManifestDraft {
    /// Ordinary decoding/capture: this does not claim bounded allocation.
    pub(crate) fn seal_ordinary(self) -> RelationalCheckpointManifest {
        RelationalCheckpointManifest(std::sync::Arc::new(ManifestBacking {
            data: self,
            _control: None,
        }))
    }
    pub(crate) fn seal_for_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointManifest, super::RelationalNativeCheckpointCaptureDenial> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ManifestBacking>())
            .map_err(|_| {
                super::RelationalNativeCheckpointCaptureDenial::Request(
                    crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                )
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| {
                super::RelationalNativeCheckpointCaptureDenial::Request(cause.into())
            })?;
        Ok(RelationalCheckpointManifest(std::sync::Arc::new(
            ManifestBacking {
                data: self,
                _control: memory,
            },
        )))
    }
}
impl RelationalCheckpointManifest {
    /// Independent legacy projection; copied heaps are explicitly ordinary.
    pub(crate) fn ordinary_owned_draft(&self) -> RelationalCheckpointManifestDraft {
        RelationalCheckpointManifestDraft {
            branch_data: self.branch_data.clone(),
            coverage: self.coverage.clone(),
            covered_position: self.covered_position,
            schemas: CheckpointSchemaImages::from_ordinary(self.schemas.ordinary_owned_copy()),
            record_identity: self.record_identity.clone(),
            aspect_contracts: self.aspect_contracts.clone(),
            index_generation_frontier: self.index_generation_frontier,
            index_definition_frontier: self.index_definition_frontier,
            index_definitions: self.index_definitions.clone(),
            roots: self.roots.clone(),
            retained_empty_components: self.retained_empty_components.clone(),
            mirror_schema: self.mirror_schema,
            mirror: self.mirror.clone(),
            history_dependencies: self.history_dependencies.clone(),
            commits: self.commits.clone(),
            canonical_members: self.canonical_members.clone(),
            symbols: self.symbols.clone(),
            symbol_identity_frontier: self.symbol_identity_frontier,
            symbol_buckets: self.symbol_buckets.clone(),
            lineage: self.lineage.clone(),
            lineage_events: self.lineage_events.clone(),
            lineage_basis: self.lineage_basis.clone(),
            derived_indexes: self.derived_indexes,
            allocations: Default::default(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RelationalCheckpointManifestDraft {
    pub(crate) branch_data: crate::branch::CheckpointBranchMetadata,
    pub(crate) coverage: CheckpointCoverage,
    pub(crate) covered_position: PatchStreamPosition,
    pub(crate) schemas: super::CheckpointSchemaImages,
    pub(crate) record_identity: DurableRecordIdentityState,
    pub(crate) aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    pub(crate) index_generation_frontier: u64,
    pub(crate) index_definition_frontier: u64,
    pub(crate) index_definitions: super::CheckpointIndexDefinitions,
    pub(crate) roots: super::CheckpointRoots,
    pub(crate) retained_empty_components: Vec<RelationalCheckpointEmptyComponent>,
    pub(crate) mirror_schema: Option<[u8; 32]>,
    pub(crate) mirror: Vec<RelationalCheckpointRegion>,
    pub(crate) history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
    pub(crate) commits: super::CheckpointCommitIndex,
    pub(crate) canonical_members:
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers,
    pub(crate) symbols: super::CheckpointPages,
    pub(crate) symbol_identity_frontier: u32,
    pub(crate) symbol_buckets: super::CheckpointSymbolBuckets,
    pub(crate) lineage: super::CheckpointPages,
    pub(crate) lineage_events: super::CheckpointLineageEventPages,
    pub(crate) lineage_basis: super::CheckpointLineageBasis,
    pub(crate) derived_indexes: RelationalDurableMemberLocator,
    pub(in crate::durability) allocations: ManifestMetadataAllocations,
}

#[derive(Debug, Default)]
pub(in crate::durability) struct ManifestMetadataAllocations {
    pub(in crate::durability) aspect_contracts: Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::durability) empty_components: Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::durability) mirror: Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::durability) _coverage: Option<worth_execution::ExecutionMemoryReservation>,
}
// Memory custody is deliberately absent from semantic equality and the wire.
impl PartialEq for ManifestMetadataAllocations {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ManifestMetadataAllocations {}

mod wire;

impl std::ops::Deref for RelationalCheckpointManifestDraft {
    type Target = crate::branch::CheckpointBranchMetadataFields;
    fn deref(&self) -> &Self::Target {
        &self.branch_data
    }
}

impl RelationalCheckpointManifestDraft {
    pub(crate) fn semantic_members(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_execution::ExecutionArray<RelationalDurableMemberLocator>,
        super::RelationalRecoveryStop,
    > {
        let mut members =
            worth_execution::ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
        let mut insert = |member| {
            members
                .insert_cloned_native_for_request(&member, &(), *request)
                .map_err(crate::durability::checkpoint_history::capture::map_stop)
        };
        for leaf in self
            .roots
            .iter()
            .flat_map(|root| &root.regions)
            .chain(&self.mirror)
        {
            insert(leaf.head)?;
            insert(leaf.history)?;
        }
        // Admitted declarations are exactly the reachable history graph.
        for dependency in &self.history_dependencies {
            insert(dependency.member)?;
            for member in &dependency.bases {
                insert(*member)?;
            }
        }
        for page in self.symbols.iter().chain(&self.lineage) {
            insert(page.member)?;
        }
        for member in self
            .symbol_buckets
            .iter()
            .flat_map(|bucket| bucket.pages.iter().copied())
        {
            insert(member)?;
        }
        for page in &self.lineage_events {
            insert(page.member)?;
        }
        for row in self.canonical_members.entries() {
            insert(row.member)?;
        }
        insert(self.derived_indexes)?;
        drop(insert);
        let mut output =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, members.len())
                .map_err(|cause| {
                    super::RelationalRecoveryStop::from(
                        crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                    )
                })?;
        for member in members.keys() {
            output.push(*member).map_err(|cause| {
                super::RelationalRecoveryStop::from(
                    crate::durability::checkpoint_history::capture::array_capture_stop(cause),
                )
            })?;
        }
        output.seal().map_err(|cause| {
            super::RelationalRecoveryStop::from(
                crate::durability::checkpoint_history::capture::array_capture_stop(cause),
            )
        })
    }

    pub(crate) fn readmit_history_members(
        &self,
    ) -> Result<
        std::sync::Arc<RelationalHistoryMemberDirectory>,
        crate::storage::data::RelationalStorageUnavailableCause,
    > {
        RelationalHistoryMemberDirectory::readmit(
            &self.history_dependencies,
            self.roots
                .iter()
                .flat_map(|root| &root.regions)
                .chain(&self.mirror)
                .map(|leaf| leaf.history),
        )
        .map(std::sync::Arc::new)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointRegion {
    pub(crate) kind_authority: crate::storage::residency::RelationalRegionKindCommitment,
    pub(crate) partition_id: PartitionId,
    pub(crate) content_digest: [u8; 32],
    pub(crate) entity_slots: usize,
    pub(crate) relation_slots: usize,
    pub(crate) unavailable_records: usize,
    pub(crate) head: RelationalDurableMemberLocator,
    pub(crate) history: RelationalDurableMemberLocator,
}

/// Value lookup has its own directory so interning a new value need not load
/// the id-ordered dictionary. Successive publications append immutable pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointSymbolBucket {
    pub(crate) bucket: u8,
    pub(crate) pages: Vec<RelationalDurableMemberLocator>,
}

/// Exact owner-recorded Empty observation with its executable schema carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RelationalCheckpointEmptyComponent {
    pub(crate) descriptor: crate::branch::RelationalBranchBasisDescriptor,
    pub(crate) schema_carrier_digest: [u8; 32],
}
