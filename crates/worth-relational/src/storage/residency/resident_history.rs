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
use crate::storage::substrate::{EntityRecordKind, RecordArena, RecordKind, RelationRecordKind};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct ResidentHistoryReadmission {
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    symbols: RuntimeSymbolTable,
}

impl ResidentHistoryReadmission {
    pub(super) fn symbols(&self) -> &RuntimeSymbolTable {
        &self.symbols
    }
    pub(super) fn new(
        schema: Arc<RelationalBranchRootSchemaAuthority>,
        symbols: &RuntimeSymbolTable,
    ) -> Self {
        Self {
            schema,
            symbols: symbols.detached_owner_snapshot(),
        }
    }

    pub(super) fn load_full(
        &self,
        head: &ResidentRegion,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
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
        let mut image = partition_to_image_with_contracts(
            head.partition.clone(),
            self.schema.aspect_plans(),
            &contracts,
        )
        .map_err(|_| refuse(Cause::Damaged))?;
        let mut folder = HistoryFolder::new(execution, partition_id, limit);
        restore_rows::<EntityRecordKind>(
            &mut folder,
            &head.partition.entity_arena,
            &mut image.entity_arena.metadata_history,
        )?;
        restore_rows::<RelationRecordKind>(
            &mut folder,
            &head.partition.relation_arena,
            &mut image.relation_arena.metadata_history,
        )?;
        let mut partition = partition_from_image(image, self.schema.aspect_plans(), &contracts)
            .map_err(|_| refuse(Cause::Damaged))?;
        partition.adjacency = head.partition.adjacency.clone();
        partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
        let projection =
            super::kind_projection::capture(&head.partition, AdjacencyKindBasis::Historical)
                .map_err(refuse)?;
        super::kind_projection::rebuild(
            &mut partition,
            &projection,
            AdjacencyKindBasis::Historical,
        )
        .map_err(refuse)?;
        self.symbols
            .demand_symbol_ids(execution, super::required_symbols::collect(&partition))?;
        let (commitment, _) = PartitionContentCommitment::capture(
            &partition,
            &self.symbols.interner_snapshot(),
            None,
        )
        .map_err(|_| refuse(Cause::Damaged))?;
        let expected = head
            .commitment
            .as_ref()
            .ok_or_else(|| refuse(Cause::Damaged))?;
        if commitment.digest(partition_id) != expected.digest(partition_id) {
            return Err(refuse(Cause::IntegrityMismatch));
        }
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        Ok(Arc::new(ResidentRegion {
            partition,
            commitment: Some(commitment),
        }))
    }
}

fn restore_rows<K: RecordKind + HistoryRowKind>(
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
            let mut earlier =
                folder.fold_row::<K>(prefix.source.as_ref(), &prefix.history_members, reference)?;
            earlier.append(suffix);
            *suffix = earlier;
        }
    }
    Ok(())
}
