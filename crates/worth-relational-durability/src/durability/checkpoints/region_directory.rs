//! Readmit a manifest's immutable region directory without reading members.
use contracts::durability::data::recovery_errors::{
    DurabilityError, DurabilityErrorOwnerPort as _,
};
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod root_readmission;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};

use crate::durability::data::RelationalCheckpointDirectory;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::identity::data::primitives::PartitionId;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::branch::root_checkpoint::RelationalRootRegionCheckpoint;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::checkpoint_region::RelationalCheckpointRegion;
use worth_relational_storage::internal_port::storage::residency::cell::{
    RelationalColdRegion, RelationalRegionResidency,
};
use worth_relational_storage::internal_port::storage::residency::kind_authority::RelationalRegionKindAuthority;

type CellKey = (
    PartitionId,
    [u8; 32],
    RelationalDurableMemberLocator,
    RelationalDurableMemberLocator,
);

pub(crate) struct ReadmittedRegionDirectory {
    pub(crate) schemas: ReadmittedSchemas,
    directory: Arc<RelationalCheckpointDirectory>,
    symbols: worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    cells: ExecutionOrderedMap<CellKey, Arc<RelationalRegionResidency>>,
    /// The runtime working set every readmitted cold cell joins, if bounded.
    working_set: Option<RegionWorkingSet>,
}

pub(crate) type RegionWorkingSet = (
    Arc<worth_relational_storage::internal_port::storage::residency::working_set::WorkingSetAccountant>,
    worth_relational_storage::internal_port::storage::residency::working_set::ResidencyPoolKind,
);

#[derive(Debug, Clone)]
pub enum ReadmittedSchemas {
    Source(
        Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
    ),
    Original(Arc<ReadmittedOriginalSchemas>),
}
#[derive(Debug)]
pub struct ReadmittedOriginalSchemas {
    rows: BTreeMap<[u8; 32], Arc<RelationalBranchRootSchemaAuthority>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ReadmittedSchemas {
    pub(crate) fn get(
        &self,
        digest: &[u8; 32],
    ) -> Option<&Arc<RelationalBranchRootSchemaAuthority>> {
        match self {
            Self::Source(catalog) => catalog.rows.get(digest),
            Self::Original(rows) => rows.rows.get(digest),
        }
    }
}

impl ReadmittedRegionDirectory {
    pub(crate) fn new(
        directory: Arc<RelationalCheckpointDirectory>,
        symbols: worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        request: &ExecutionRequest<'_, '_>,
        configured_schema: &contracts::schema::data::RelationalSchemaRegistry,
    ) -> Result<Self, RelationalRecoveryStop> {
        if let Some(catalog) = directory.source_schemas.get() {
            let schemas = ReadmittedSchemas::Source(catalog.clone());
            return Ok(Self {
                schemas,
                symbols: symbols.from_selected_directory_for_request(
                    directory.manifest.symbol_page_roster(),
                    directory.symbol_member_custody(),
                    request,
                )?,
                directory,
                cells: Default::default(),
                working_set: None,
            });
        }
        if directory.admission_image.is_some() {
            return Err(corrupt(
                "checkpoint image requires its actual admitted source schema catalog",
            )
            .into());
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ReadmittedOriginalSchemas>())
            .map_err(|_| corrupt("readmitted schema directory layout overflow"))?
            .0
            .pad_to_align();
        let count = directory.manifest.schemas.len();
        let nodes = if count == 0 {
            0
        } else {
            contracts::branch::root::schema::standard_tree_quote::retained_tree_bytes::<
                [u8; 32],
                Arc<RelationalBranchRootSchemaAuthority>,
            >(count)
            .map_err(|_| corrupt("readmitted schema map layout overflow"))?
        };
        let bytes = nodes
            .checked_add(layout.size() as u64)
            .ok_or_else(|| corrupt("readmitted schema directory layout overflow"))?;
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
        let configured_snapshot = (!directory.manifest.schemas.is_empty())
            .then(|| {
                configured_schema
                    .snapshot_for_compilation(
                        contracts::schema::compilation_control::SchemaCompilationControl::Request(
                            *request,
                        ),
                    )
                    .map_err(RelationalRecoveryStop::Request)
            })
            .transpose()?;
        let mut schemas = BTreeMap::new();
        for (index, image) in directory.manifest.schemas.iter().enumerate() {
            let (registry, contracts) = image.readmit_registry().map_err(corrupt)?;
            let expected = registry
                .snapshot_for_compilation(
                    contracts::schema::compilation_control::SchemaCompilationControl::Request(
                        *request,
                    ),
                )
                .map_err(RelationalRecoveryStop::Request)?;
            // Preserve genuine installed recipe/domain issuers when the exact
            // schema meaning matches the receiving runtime's declaration.
            let mut registry = if configured_snapshot.as_ref() == Some(&expected) {
                configured_schema.clone()
            } else {
                registry
            };
            (*registry.owner_content_formats_mut()) = (*registry.owner_content_formats())
                .bind_recorded(configured_schema.owner_content_formats())
                .map_err(RelationalRecoveryStop::from)?;
            let authority = RelationalBranchRootSchemaAuthority::readmit_exact(
                index as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
                contracts::schema::compilation_control::SchemaCompilationControl::Request(*request),
            )
            .map_err(RelationalRecoveryStop::Request)?
            .ok_or_else(|| corrupt("schema carrier executable readmission failed"))?;
            if schemas.insert(image.carrier_digest(), authority).is_some() {
                return Err(corrupt("duplicate schema carrier").into());
            }
        }
        Ok(Self {
            schemas: ReadmittedSchemas::Original(Arc::new(ReadmittedOriginalSchemas {
                rows: schemas,
                _memory: memory,
            })),
            directory,
            // Cells retain an isolated readmission dictionary, never a live
            // runtime's writable symbol authority. Serving leases readmit into
            // the demanding runtime separately, including after a fork.
            symbols: symbols.detached_owner_snapshot(),
            cells: Default::default(),
            working_set: None,
        })
    }

    pub(crate) fn from_readmitted_schemas(
        directory: Arc<RelationalCheckpointDirectory>,
        symbols: worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        schemas: ReadmittedSchemas,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        contracts::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        Ok(Self {
            directory,
            symbols,
            schemas,
            cells: Default::default(),
            working_set: None,
        })
    }
    /// Bind every cold cell this directory readmits to the pool of `accountant`.
    pub(crate) fn with_working_set(mut self, working_set: Option<RegionWorkingSet>) -> Self {
        self.working_set = working_set;
        self
    }

    pub(crate) fn regions(
        &mut self,
        schema_digest: [u8; 32],
        leaves: &[RelationalCheckpointRegion],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionArray<RelationalRootRegionCheckpoint>, RelationalRecoveryStop> {
        let schema = self
            .schemas
            .get(&schema_digest)
            .cloned()
            .ok_or_else(|| corrupt("region directory names a missing schema carrier"))?;
        let authority = RelationalRegionKindAuthority::for_request(request)?;
        let mut result = ExecutionRequestArrayBuilder::allocate(*request, leaves.len()).map_err(
            contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
        )?;
        for leaf in leaves {
            contracts::durability::data::recovery_stop::account_work(request, 1)?;
            let key = (leaf.partition_id, schema_digest, leaf.head, leaf.history);
            let cell = match self.cells.get(&key) {
                Some(cell) => cell.clone(),
                None => {
                    let cell = RelationalRegionResidency::from_selected_directory(
                        RelationalColdRegion {
                            readmission: &crate::storage::residency::region_load::CHECKPOINT_REGION_READMISSION,
                            kind_authority: leaf.kind_authority,
                            source: Some(self.directory.source.clone()),
                            partition_id: leaf.partition_id,
                            head: leaf.head,
                            history: leaf.history,
                            history_members: self.directory.history_members.clone(),
                            schema: schema.clone(),
                            schema_carrier_digest: schema_digest,
                            symbols: self.symbols.clone(),
                            content_digest: leaf.content_digest,
                            directory: leaf.directory.clone(),
                            live_counts: leaf.live_counts,
                            entity_slots: leaf.entity_slots,
                            relation_slots: leaf.relation_slots,
                            unavailable_records: leaf.unavailable_records,
                            chunks: leaf.chunks.clone(),
                        },
                        self.directory.region_member_basis(),
                        &self.directory.history_members,
                        request,
                    )?;
                    if let Some((accountant, pool)) = &self.working_set {
                        // Fresh cold cells are backed and unbound: binding cannot refuse.
                        let _ = cell.bind_working_set(accountant, *pool);
                    }
                    self.cells
                        .insert_cloned_native_for_request(&key, &cell, *request)
                        .map_err(contracts::durability::data::recovery_stop::map_stop)?;
                    cell
                }
            };
            // G7: only the fixed-size descriptor is installed; directory
            // pages fault on demand through the cell's working set.
            let record_directory = cell.record_directory(leaf.directory.clone());
            result
                .push(RelationalRootRegionCheckpoint {
                    partition_id: leaf.partition_id,
                    content_digest: leaf.content_digest,
                    kind_commitment: leaf.kind_authority,
                    directory: record_directory,
                    live_counts: leaf.live_counts,
                    entity_slots: leaf.entity_slots,
                    relation_slots: leaf.relation_slots,
                    unavailable_records: leaf.unavailable_records,
                    cell,
                    kind_authority: Some(authority.clone()),
                })
                .map_err(contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop)?;
        }
        let result = result.seal().map_err(
            contracts::durability::data::native_checkpoint_capture_denial::array_capture_stop,
        )?;
        contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane::enter_reconstruction(|lane| {
            authority.bind_for_request(
                lane,
                result
                    .iter()
                    .map(|region| (region.partition_id, region.cell.clone())),
                request,
            )
        })?;
        Ok(result)
    }
}

pub fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::corrupt_checkpoint(detail)
}
