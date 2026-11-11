//! Independent Full reconstruction of an owned Head and its cold row bases.
use super::history_fold::{FundedRows, HistoryFolder, HistoryRowKind};
use super::region_image::RegionHistoryBase;
use super::region_load::materialization_funding as funding;
use super::{RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image_for_request, partition_to_image_with_contracts,
};
use crate::runtime::RuntimeSymbolTable;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::storage::substrate::{EntityRecordKind, RecordArena, RecordKind, RelationRecordKind};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionRequest;
mod projection_capture;

#[derive(Debug, Clone)]
pub(crate) struct ResidentHistoryReadmission {
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    symbols: RuntimeSymbolTable,
    pub(super) publication: Option<super::RegionSymbolPublication>,
}

impl ResidentHistoryReadmission {
    pub(super) fn symbols(&self) -> &RuntimeSymbolTable {
        &self.symbols
    }
    pub(crate) fn new(
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        symbols: &RuntimeSymbolTable,
    ) -> Self {
        Self {
            schema,
            publication: None,
            symbols: symbols.detached_owner_snapshot(),
        }
    }

    pub(crate) fn with_symbol_publication(
        mut self,
        publication: super::RegionSymbolPublication,
    ) -> Self {
        self.publication = Some(publication);
        self
    }
    pub(super) fn load_full(
        &self,
        head: &ResidentRegion,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        ReconstructionLane::enter_reconstruction(|lane| {
            let partition_id = head.partition.partition_id;
            let refuse = |cause| {
                Failure::StorageUnavailable(RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Region(partition_id),
                    cause,
                })
            };
            let (_catalog_memory, _) = funding::contracts(&self.schema, execution)?;
            let (_projection_memory, incidences) = projection_funding(&head.partition, execution)?;
            let current_quote = funding::current(
                &head.partition,
                self.schema.retained_aspect_contracts(),
                execution,
            )?;
            let _current_wire = funding::Funding::reserve(current_quote.wire, execution)?;
            let contracts = CheckpointAspectContractCatalog::from_contracts(
                self.schema.retained_aspect_contracts(),
            )
            .map_err(|_| refuse(Cause::Damaged))?;
            // Owned current edits have no required acceleration cache. Derive
            // their expected root from Head and authenticated prefix openings
            // before reading any historical member, independently of folding.
            self.symbols.demand_symbol_ids(
                execution,
                super::required_symbols::collect_for_request(
                    lane,
                    &head.partition,
                    false,
                    execution,
                )?
                .ids,
            )?;
            let expected = match &head.commitment {
                Some(commitment) => commitment.clone(),
                None => {
                    PartitionContentCommitment::capture_layered(
                        lane,
                        execution,
                        &head.partition,
                        &self.symbols.interner_snapshot(),
                    )
                    .map_err(|cause| funding::content(cause, partition_id))?
                    .0
                }
            };
            let mut image = partition_to_image_with_contracts(
                lane,
                head.partition.clone(),
                self.schema.aspect_plans(),
                &contracts,
            )
            .map_err(|_| refuse(Cause::Damaged))?;
            let mut folder = HistoryFolder::new(lane, execution, partition_id, limit);
            let _entity_rows = restore_rows::<EntityRecordKind>(
                lane,
                &mut folder,
                &head.partition.entity_arena,
                &mut image.entity_arena.metadata_history,
                execution,
            )?;
            let _relation_rows = restore_rows::<RelationRecordKind>(
                lane,
                &mut folder,
                &head.partition.relation_arena,
                &mut image.relation_arena.metadata_history,
                execution,
            )?;
            let memory =
                funding::Funding::reserve(funding::shared::<ResidentRegion>()?, execution)?;
            let mut partition = partition_from_image_for_request(
                lane,
                image,
                self.schema.aspect_plans(),
                &contracts,
                execution,
            )
            .map_err(|cause| funding::image(cause, partition_id))?;
            partition.adjacency = head.partition.adjacency.clone();
            partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
            let projection = if head.kind_indexes.is_some() {
                projection_capture::capture(lane, &head.partition, incidences, execution)?
            } else {
                super::kind_admission::recover(lane, execution, partition_id, |checkpoint, _| {
                    super::kind_projection::canonical_history_controlled(
                        lane, &partition, checkpoint,
                    )
                })?
            };
            super::kind_projection::rebuild_for_request(
                lane,
                &mut partition,
                &projection,
                AdjacencyKindBasis::Historical,
                execution,
            )?;
            let requirements =
                super::required_symbols::CapturedSymbolRequirements::capture_for_request(
                    lane, &partition, true, execution,
                )?;
            self.symbols.demand_symbol_ids(
                execution,
                requirements.get(true).unwrap().ids.iter().copied(),
            )?;
            let (commitment, _) = PartitionContentCommitment::capture(
                lane,
                execution,
                &partition,
                &self.symbols.interner_snapshot(),
            )
            .map_err(|cause| funding::content(cause, partition_id))?;
            if commitment.digest(partition_id) != expected.digest(partition_id) {
                return Err(refuse(Cause::IntegrityMismatch));
            }
            crate::execution::check_request(execution).map_err(Failure::Request)?;
            Ok(Arc::new(ResidentRegion {
                symbol_requirements: OnceLock::from(Arc::new(requirements)),
                history_presence: super::history_presence::HistoryPresence::complete(),
                kind_indexes: Some(super::kind_admission::reconstruct(
                    lane, execution, &partition,
                )?),
                partition,
                commitment: Some(commitment),
                _history_custody: None,
                _memory: memory.into_memory(),
            }))
        })
    }
}

fn restore_rows<K: RecordKind + HistoryRowKind>(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    folder: &mut HistoryFolder<'_, '_, '_>,
    arena: &RecordArena<K>,
    rows: &mut [Vec<K::Metadata>],
    execution: &ExecutionRequest<'_, '_>,
) -> Result<FundedRows<K::Metadata>, Failure> {
    let mut custody = FundedRows::allocate(rows.len(), execution)?;
    for (physical, suffix) in rows.iter_mut().enumerate() {
        for metadata in suffix.iter() {
            folder.charge_output(metadata)?;
        }
        if let Some(prefix) = &arena.history_prefixes[physical] {
            let reference = RegionHistoryBase {
                member: prefix.member,
                partition_id: prefix.partition_id,
                schema_carrier_digest: prefix.schema_carrier_digest,
                row_index: prefix.row_index as u64,
                kind_commitment: prefix.history_opening.kind_commitment,
            };
            let mut earlier = folder.fold_row::<K>(
                lane,
                prefix
                    .source()
                    .map_err(|cause| {
                        Failure::StorageUnavailable(RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Region(prefix.partition_id),
                            cause,
                        })
                    })?
                    .as_ref(),
                &prefix.history_members,
                reference,
            )?;
            earlier.append_current(suffix, execution)?;
            *suffix = std::mem::take(&mut earlier.values);
            custody.rows.push(earlier);
        }
    }
    Ok(custody)
}

pub(super) fn projection_funding(
    partition: &PartitionState,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(funding::Funding, usize), Failure> {
    let _cursors = funding::Funding::reserve(
        funding::vector::<[usize; 2]>(8 * usize::BITS as usize)?,
        request,
    )?;
    let mut incidences = 0usize;
    for (_, adjacency) in partition
        .adjacency
        .iter()
        .chain(partition.reverse_adjacency.iter())
    {
        for _ in adjacency.kind_memberships(AdjacencyKindBasis::Historical) {
            funding::work(request, 1)?;
            incidences = incidences.checked_add(1).ok_or_else(funding::overflow)?;
        }
    }
    let bytes = funding::sum(
        funding::region_image::projection_storage(incidences, incidences)?.wire,
        funding::vector::<[usize; 2]>(8 * usize::BITS as usize)?,
    )?;
    // This is simultaneous backing, not work. Reconstruction meters its own
    // operations; owned-Head capture has a separate original-request preflight.
    Ok((funding::Funding::reserve(bytes, request)?, incidences))
}
