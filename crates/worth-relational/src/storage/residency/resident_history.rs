//! Independent Full reconstruction of an owned Head and its cold row bases.
use super::history_fold::{HistoryFolder, HistoryRowKind};
use super::region_image::RegionHistoryBase;
use super::{RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image, partition_to_image_with_contracts,
};
use crate::runtime::RuntimeSymbolTable;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::PartitionContentCommitment;
use crate::storage::partition::AdjacencyKindBasis;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::storage::substrate::{EntityRecordKind, RecordArena, RecordKind, RelationRecordKind};
use std::sync::{Arc, OnceLock};

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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
            let contracts = CheckpointAspectContractCatalog::from_contracts(
                self.schema.retained_aspect_contracts(),
            )
            .map_err(|_| refuse(Cause::Damaged))?;
            // Owned current edits have no required acceleration cache. Derive
            // their expected root from Head and authenticated prefix openings
            // before reading any historical member, independently of folding.
            self.symbols.demand_symbol_ids(
                execution,
                super::required_symbols::collect_head(lane, &head.partition),
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
                    .map_err(|error| super::content_capture::restore_failure(partition_id, error))?
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
            restore_rows::<EntityRecordKind>(
                lane,
                &mut folder,
                &head.partition.entity_arena,
                &mut image.entity_arena.metadata_history,
            )?;
            restore_rows::<RelationRecordKind>(
                lane,
                &mut folder,
                &head.partition.relation_arena,
                &mut image.relation_arena.metadata_history,
            )?;
            let mut partition =
                partition_from_image(lane, image, self.schema.aspect_plans(), &contracts)
                    .map_err(|_| refuse(Cause::Damaged))?;
            partition.adjacency = head.partition.adjacency.clone();
            partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
            let projection = if head.kind_indexes.is_some() {
                // Published Head carries authenticated kind authority. Its
                // supplied claims must still equal independently restored rows.
                super::kind_projection::capture(
                    lane,
                    &head.partition,
                    AdjacencyKindBasis::Historical,
                )
            } else {
                // The mutable mirror has no committed kind authority. Retention
                // and slot reuse can discard owned generations while their
                // optional adjacency cache still names them. Reconstruct local
                // claims from restored truth; peer claims stay carried, and the
                // independent content commitment below checks all owned bytes.
                super::kind_projection::canonical_history(lane, &partition)
            }
            .map_err(refuse)?;
            super::kind_projection::rebuild(
                lane,
                &mut partition,
                &projection,
                AdjacencyKindBasis::Historical,
            )
            .map_err(refuse)?;
            let requirements = super::required_symbols::CapturedSymbolRequirements::capture(
                lane, &partition, true,
            );
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
            .map_err(|error| super::content_capture::restore_failure(partition_id, error))?;
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
            }))
        })
    }
}

fn restore_rows<K: RecordKind + HistoryRowKind>(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    folder: &mut HistoryFolder<'_, '_, '_>,
    arena: &RecordArena<K>,
    rows: &mut [Vec<K::Metadata>],
) -> Result<(), Failure> {
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
                prefix.source.as_ref(),
                &prefix.history_members,
                reference,
            )?;
            earlier.append(suffix);
            *suffix = earlier;
        }
    }
    Ok(())
}
