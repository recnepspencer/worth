//! The eager directory of the current native checkpoint. Record payloads,
//! history, and rebuildable projections belong to members, never this directory.
use serde::{Deserialize, Serialize};
use worth_relational_contracts::internal_port as contracts;
pub(crate) use worth_relational_storage::internal_port::storage::data::checkpoint_region::RelationalCheckpointRegion;

use super::*;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;

#[derive(Debug, Clone)]
pub struct RelationalCheckpointManifest(std::sync::Arc<ManifestBacking>);
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
    pub fn seal_ordinary(self) -> RelationalCheckpointManifest {
        RelationalCheckpointManifest(std::sync::Arc::new(ManifestBacking {
            data: self,
            _control: None,
        }))
    }
    pub(crate) fn seal_for_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalCheckpointManifest, contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial>{
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ManifestBacking>())
            .map_err(|_| {
                contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::Request(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::MemoryOverflow,
                )
            })?
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| {
                contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::Request(cause.into())
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
    pub fn ordinary_owned_draft(&self) -> RelationalCheckpointManifestDraft {
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
pub struct RelationalCheckpointManifestDraft {
    pub(crate) branch_data: worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata,
    pub(crate) coverage: CheckpointCoverage,
    pub covered_position: PatchStreamPosition,
    pub schemas: super::CheckpointSchemaImages,
    pub record_identity: DurableRecordIdentityState,
    pub(crate) aspect_contracts: Vec<worth_foundational::facade::PortableAspectContract>,
    pub(crate) index_generation_frontier: u64,
    pub index_definition_frontier: u64,
    pub(crate) index_definitions: super::CheckpointIndexDefinitions,
    pub roots: super::CheckpointRoots,
    pub retained_empty_components: Vec<RelationalCheckpointEmptyComponent>,
    pub(crate) mirror_schema: Option<[u8; 32]>,
    pub mirror: Vec<RelationalCheckpointRegion>,
    pub(crate) history_dependencies: Vec<RelationalCheckpointHistoryDependency>,
    pub commits: super::CheckpointCommitIndex,
    pub(crate) canonical_members:
        crate::durability::checkpoint_history::inventory::CanonicalCheckpointMembers,
    pub symbols: super::CheckpointPages,
    pub symbol_identity_frontier: u32,
    pub symbol_buckets: super::CheckpointSymbolBuckets,
    pub lineage: super::CheckpointPages,
    pub lineage_events: super::CheckpointLineageEventPages,
    pub(crate) lineage_basis: super::CheckpointLineageBasis,
    pub derived_indexes: RelationalDurableMemberLocator,
    pub(in crate::internal_port::durability) allocations: ManifestMetadataAllocations,
}

#[derive(Debug, Default)]
pub(in crate::internal_port::durability) struct ManifestMetadataAllocations {
    pub(in crate::internal_port::durability) aspect_contracts:
        Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::internal_port::durability) empty_components:
        Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::internal_port::durability) mirror:
        Option<worth_execution::ExecutionMemoryReservation>,
    pub(in crate::internal_port::durability) _coverage:
        Option<worth_execution::ExecutionMemoryReservation>,
}
// Memory custody is deliberately absent from semantic equality and the wire.
impl PartialEq for ManifestMetadataAllocations {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl Eq for ManifestMetadataAllocations {}

pub mod wire;

impl std::ops::Deref for RelationalCheckpointManifestDraft {
    type Target = worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadataFields;
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
        contracts::durability::data::recovery_stop::RelationalRecoveryStop,
    > {
        let mut members =
            worth_execution::ExecutionOrderedMap::<RelationalDurableMemberLocator, ()>::default();
        let mut insert = |member| {
            members
                .insert_cloned_native_for_request(&member, &(), *request)
                .map_err(contracts::durability::data::recovery_stop::map_stop)
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
                    contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(
                contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(
                    cause,
                ),
            )
                })?;
        for member in members.keys() {
            output.push(*member).map_err(|cause| {
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(
                    contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(cause),
                )
            })?;
        }
        output.seal().map_err(|cause| {
            contracts::durability::data::recovery_stop::RelationalRecoveryStop::from(
                contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop(
                    cause,
                ),
            )
        })
    }

    pub fn readmit_history_members(
        &self,
        lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    ) -> Result<
        std::sync::Arc<RelationalHistoryMemberDirectory>,
        contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
        RelationalHistoryMemberDirectory::readmit(
            lane,
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

/// Exact owner-recorded Empty observation with its executable schema carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointEmptyComponent {
    pub(crate) descriptor: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    pub(crate) schema_carrier_digest: [u8; 32],
}

/// The Native identity and the physical member locator are checked independently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationalCheckpointContent {
    pub(crate) identity: worth_foundational::facade::ContentRefId,
    pub(crate) member: RelationalDurableMemberLocator,
}

impl RelationalCheckpointManifest {
    pub fn symbol_page_roster(
        &self,
    ) -> worth_relational_storage::internal_port::storage::data::symbol_page_roster::SymbolPageRoster<'_>{
        worth_relational_storage::internal_port::storage::data::symbol_page_roster::SymbolPageRoster {
            by_id: &self.symbols,
            by_value: &self.symbol_buckets,
            frontier: self.symbol_identity_frontier,
        }
    }
}
