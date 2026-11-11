//! Readmit a manifest's immutable region directory without reading members.
mod root_readmission;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};

use crate::branch::{RelationalBranchRootSchemaAuthority, RelationalRootRegionCheckpoint};
use crate::durability::data::{
    DurabilityError, RecoveryFailureClass, RelationalCheckpointDirectory,
    RelationalCheckpointRegion, RelationalRecoveryStop,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::PartitionId;
use crate::storage::residency::{
    RelationalColdRegion, RelationalRegionKindAuthority, RelationalRegionResidency,
};

type CellKey = (
    PartitionId,
    [u8; 32],
    RelationalDurableMemberLocator,
    RelationalDurableMemberLocator,
);

pub(crate) struct ReadmittedRegionDirectory {
    pub(crate) schemas: ReadmittedSchemas,
    directory: Arc<RelationalCheckpointDirectory>,
    symbols: crate::runtime::RuntimeSymbolTable,
    cells: ExecutionOrderedMap<CellKey, Arc<RelationalRegionResidency>>,
}

#[derive(Debug, Clone)]
pub(crate) enum ReadmittedSchemas {
    Source(
        Arc<
            crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog,
        >,
    ),
    Original(Arc<ReadmittedOriginalSchemas>),
}
#[derive(Debug)]
pub(crate) struct ReadmittedOriginalSchemas {
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
        symbols: crate::runtime::RuntimeSymbolTable,
        request: &ExecutionRequest<'_, '_>,
        content_formats: &crate::content::RelationalContentFormatRegistry,
    ) -> Result<Self, RelationalRecoveryStop> {
        if let Some(catalog) = directory.source_schemas.get() {
            let schemas = ReadmittedSchemas::Source(catalog.clone());
            return Ok(Self {
                schemas,
                symbols: symbols.from_selected_directory_for_request(&directory, request)?,
                directory,
                cells: Default::default(),
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
            crate::branch::retained_compilation_tree_bytes::<
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
        let mut schemas = BTreeMap::new();
        for (index, image) in directory.manifest.schemas.iter().enumerate() {
            let (mut registry, contracts) = image.readmit_registry().map_err(corrupt)?;
            registry.content_formats = registry
                .content_formats
                .bind_recorded(content_formats)
                .map_err(RelationalRecoveryStop::from)?;
            let expected = registry
                .snapshot_for_compilation(crate::schema::SchemaCompilationControl::Request(
                    *request,
                ))
                .map_err(RelationalRecoveryStop::Request)?;
            let authority = RelationalBranchRootSchemaAuthority::readmit_exact(
                index as u64 + 1,
                registry,
                contracts,
                &expected,
                image.descriptor_semantics_version(),
                crate::schema::SchemaCompilationControl::Request(*request),
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
        })
    }

    pub(crate) fn from_readmitted_schemas(
        directory: Arc<RelationalCheckpointDirectory>,
        symbols: crate::runtime::RuntimeSymbolTable,
        schemas: ReadmittedSchemas,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalRecoveryStop> {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
        Ok(Self {
            directory,
            symbols,
            schemas,
            cells: Default::default(),
        })
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
        let mut result = ExecutionRequestArrayBuilder::allocate(*request, leaves.len())
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        for leaf in leaves {
            let record_directory =
                crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                    |lane| {
                        lane.recover(request, |lane, work| {
                            crate::storage::residency::RegionRecordDirectory::from_portable(
                                lane,
                                leaf.partition_id,
                                leaf.directory.clone(),
                                &mut |units, bytes| work.prepare(units, bytes).map_err(Into::into),
                            )
                        })
                    },
                )
                .map_err(
                    |cause: crate::storage::residency::RecordDirectoryError| {
                        cause.into_failure(leaf.partition_id)
                    },
                )?;
            crate::durability::artifact_payload_emission::request::account_work(request, 1)?;
            let key = (leaf.partition_id, schema_digest, leaf.head, leaf.history);
            let cell = match self.cells.get(&key) {
                Some(cell) => cell.clone(),
                None => {
                    let cell = RelationalRegionResidency::from_selected_directory(
                        RelationalColdRegion {
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
                        },
                        &self.directory,
                        request,
                    )?;
                    self.cells
                        .insert_cloned_native_for_request(&key, &cell, *request)
                        .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
                    cell
                }
            };
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
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
        let result = result
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
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

pub(crate) fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
