//! Independent Full reconstruction of an owned Head and its cold row bases.
use super::history_fold::{FundedRows, HistoryFolder, HistoryRowKind};
use super::region_image::RegionHistoryBase;
use super::region_load::materialization_funding as funding;
use super::{RelationalResidencyDemandFailure as Failure, ResidentRegion};
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image, partition_to_image_with_contracts,
};
use crate::identity::data::{KindId, RelationId, VersionId};
use crate::runtime::RuntimeSymbolTable;
use crate::storage::data::{RecordLifecycleState, RelationalFieldRevision};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::overlay::{PartitionContentCommitment, PartitionState};
use crate::storage::partition::{AdjacencyKindBasis, AdjacencySet};
use crate::storage::substrate::{AspectVersions, FieldRevisions};
use crate::storage::substrate::{
    EntityExtra, EntityRecordKind, RecordArena, RecordKind, RelationExtra, RelationRecordKind,
    VersionedEntityMetadata, VersionedRelationMetadata,
};
use crate::symbols::data::Symbol;
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

#[derive(Debug, Clone)]
pub(crate) struct ResidentHistoryReadmission {
    schema: Arc<RelationalBranchRootSchemaAuthority>,
    symbols: RuntimeSymbolTable,
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
            symbols: symbols.detached_owner_snapshot(),
        }
    }

    pub(super) fn load_full(
        &self,
        head: &ResidentRegion,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<Arc<ResidentRegion>, Failure> {
        let partition_id = head.partition.partition_id;
        let refuse = |cause| {
            Failure::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(partition_id),
                cause,
            })
        };
        let (_catalog_memory, contract_bound) = funding::contracts(&self.schema, execution)?;
        let (_projection_memory, incidences) = projection_funding(&head.partition, execution)?;
        let current_quote = funding::current(&head.partition, contract_bound, execution)?;
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
            super::required_symbols::collect_head(&head.partition),
        )?;
        let mut expected_memory = funding::Funding::default();
        if head.commitment.is_none() {
            let e = &head.partition.entity_arena;
            let r = &head.partition.relation_arena;
            let mut nodes = e
                .generations
                .len()
                .checked_add(r.generations.len())
                .and_then(|n| n.checked_mul(5))
                .ok_or_else(funding::overflow)?;
            for prefix in e
                .history_prefixes
                .iter()
                .chain(r.history_prefixes.iter())
                .flatten()
            {
                funding::work(execution, 1)?;
                nodes = nodes
                    .checked_add(prefix.history_opening.frontier.siblings.len())
                    .ok_or_else(funding::overflow)?;
            }
            let bytes = funding::region_image::head_commitment_nodes(nodes)?;
            funding::work(execution, bytes)?;
            expected_memory.grow(bytes, execution)?;
        }
        let expected = match &head.commitment {
            Some(commitment) => commitment.clone(),
            None => {
                PartitionContentCommitment::capture_layered(
                    &head.partition,
                    &self.symbols.interner_snapshot(),
                )
                .map_err(|_| refuse(Cause::Damaged))?
                .0
            }
        };
        let mut image = partition_to_image_with_contracts(
            head.partition.clone(),
            self.schema.aspect_plans(),
            &contracts,
        )
        .map_err(|_| refuse(Cause::Damaged))?;
        let mut folder = HistoryFolder::new(execution, partition_id, limit);
        let _entity_rows = restore_rows::<EntityRecordKind>(
            &mut folder,
            &head.partition.entity_arena,
            &mut image.entity_arena.metadata_history,
            execution,
        )?;
        let _relation_rows = restore_rows::<RelationRecordKind>(
            &mut folder,
            &head.partition.relation_arena,
            &mut image.relation_arena.metadata_history,
            execution,
        )?;
        let memory = fund_native_image(&image, contract_bound, incidences, execution)?;
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
        let (commitment, _) = PartitionContentCommitment::capture_full_for_request(
            &partition,
            &self.symbols.interner_snapshot(),
            execution,
        )
        .map_err(|cause| funding::content(cause, partition_id))?;
        if commitment.digest(partition_id) != expected.digest(partition_id) {
            return Err(refuse(Cause::IntegrityMismatch));
        }
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        Ok(Arc::new(ResidentRegion {
            partition,
            commitment: Some(commitment),
            _history_custody: None,
            _memory: memory.into_memory(),
        }))
    }
}

fn restore_rows<K: RecordKind + HistoryRowKind>(
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

/// Preflight the completed wire image before native readmission allocates.
pub(super) fn fund_native_image(
    image: &crate::durability::data::PartitionCheckpointImage,
    contract: u64,
    historical_incidences: usize,
    request: &ExecutionRequest<'_, '_>,
) -> Result<funding::Funding, Failure> {
    use crate::storage::substrate::{SharedColumn, StorageAllocation};
    use funding::{self as fund, aspect_state as aspects};
    let e = &image.entity_arena;
    let r = &image.relation_arena;
    let mut bytes = fund::shared::<ResidentRegion>()?;
    for count in [e.generations.len(), r.generations.len()] {
        bytes = fund::sum(bytes, native_scalar_columns(count)?)?;
    }
    bytes = fund::sum(
        bytes,
        fund::column::<SharedColumn<StorageAllocation<VersionedEntityMetadata>>>(
            e.metadata_history.len(),
        )?,
    )?;
    bytes = fund::sum(
        bytes,
        fund::column::<SharedColumn<StorageAllocation<VersionedRelationMetadata>>>(
            r.metadata_history.len(),
        )?,
    )?;
    bytes = fund::sum(
        bytes,
        fund::column::<StorageAllocation<EntityExtra>>(e.extra.len())?,
    )?;
    bytes = fund::sum(
        bytes,
        fund::column::<StorageAllocation<RelationExtra>>(r.extra.len())?,
    )?;
    for rows in &e.metadata_history {
        bytes = fund::sum(
            bytes,
            fund::column::<StorageAllocation<VersionedEntityMetadata>>(rows.len())?,
        )?;
        for meta in rows {
            bytes = fund::sum(
                bytes,
                fund::aspect_state::entity_metadata(meta, contract)?.native,
            )?;
        }
    }
    for rows in &r.metadata_history {
        bytes = fund::sum(
            bytes,
            fund::column::<StorageAllocation<VersionedRelationMetadata>>(rows.len())?,
        )?;
        for meta in rows {
            bytes = fund::sum(
                bytes,
                fund::aspect_state::relation_metadata(meta, contract)?.native,
            )?;
        }
    }
    for extra in &e.extra {
        bytes = fund::sum(
            bytes,
            aspects::owned(extra.authoritative_aspect_state.as_ref(), contract)?.native,
        )?;
    }
    for extra in &r.extra {
        bytes = fund::sum(
            bytes,
            aspects::owned(extra.authoritative_aspect_state.as_ref(), contract)?.native,
        )?;
    }
    for maps in [&e.aspect_versions, &r.aspect_versions] {
        bytes = fund::sum(bytes, fund::column::<AspectVersions>(maps.len())?)?;
        for row in maps {
            bytes = fund::sum(bytes, fund::map::<Symbol, u64>(row.len())?)?;
        }
    }
    for maps in [&e.field_revisions, &r.field_revisions] {
        bytes = fund::sum(bytes, fund::column::<Option<FieldRevisions>>(maps.len())?)?;
        for row in maps.iter().flatten() {
            bytes = fund::sum(
                bytes,
                fund::map::<(Symbol, Symbol), RelationalFieldRevision>(row.len())?,
            )?;
        }
    }
    for maps in [&e.diagnostics_enrichment, &r.diagnostics_enrichment] {
        bytes = fund::sum(
            bytes,
            fund::column::<StorageAllocation<BTreeMap<Symbol, String>>>(maps.len())?,
        )?;
        for row in maps {
            bytes = fund::sum(bytes, fund::tree::<Symbol, String>(row.len())?)?;
            for value in row.values() {
                bytes = fund::sum(bytes, value.len() as u64)?;
            }
        }
    }
    for pins in [
        &e.branch_pins,
        &e.replay_pins,
        &e.snapshot_pins,
        &r.branch_pins,
        &r.replay_pins,
        &r.snapshot_pins,
    ] {
        bytes = fund::sum(
            bytes,
            fund::map::<usize, u32>(pins.iter().filter(|value| **value != 0).count())?,
        )?;
    }
    for bits in [
        &e.live_bitset,
        &e.reclaimable_bitset,
        &r.live_bitset,
        &r.reclaimable_bitset,
    ] {
        bytes = fund::sum(
            bytes,
            fund::map::<usize, u64>(bits.words.len() + bits.sparse_words.len())?,
        )?;
    }
    for table in [&image.adjacency, &image.reverse_adjacency] {
        bytes = fund::sum(bytes, fund::map::<usize, AdjacencySet>(table.len())?)?;
        for entry in table {
            bytes = fund::sum(bytes, fund::map::<RelationId, ()>(entry.relations.len())?)?;
            bytes = fund::sum(
                bytes,
                fund::map::<KindId, VersionId>(entry.structural_revisions.len())?,
            )?;
        }
    }
    // Include independently retained peer claims as well as local row history.
    // Invalid supplied claims can still require local reconstruction before refusal.
    let incidence = r
        .metadata_history
        .iter()
        .try_fold(historical_incidences, |n, rows| {
            n.checked_add(rows.len().checked_mul(2)?)
        })
        .ok_or_else(fund::overflow)?;
    bytes = fund::sum(
        bytes,
        fund::region_image::projection_storage(incidence, incidence)?.native,
    )?;
    fund::work(request, bytes)?;
    fund::Funding::reserve(bytes, request)
}
fn native_scalar_columns(n: usize) -> Result<u64, Failure> {
    use super::region_load::materialization_funding as f;
    use crate::identity::data::{KindId, PartitionId, VersionId};
    use crate::storage::substrate::{RecordHistoryPrefix, StorageAllocation};
    let rows = [
        f::column::<PartitionId>(n)?,
        f::column::<u32>(n)?,
        f::column::<RecordLifecycleState>(n)?,
        f::column::<Option<KindId>>(n)?,
        f::column::<u64>(n)?,
        f::map::<u64, usize>(n)?,
        f::column::<VersionId>(n)?,
        f::column::<Option<VersionId>>(n)?,
        f::column::<StorageAllocation<Option<RecordHistoryPrefix>>>(n)?,
    ];
    rows.into_iter().try_fold(0, f::sum)
}

pub(super) fn projection_funding(
    partition: &PartitionState,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(funding::Funding, usize), Failure> {
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
    let bytes = funding::region_image::projection_storage(incidences, incidences)?.wire;
    funding::work(request, bytes)?;
    Ok((funding::Funding::reserve(bytes, request)?, incidences))
}
