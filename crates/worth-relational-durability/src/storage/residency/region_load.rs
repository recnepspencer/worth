pub mod head_validation;
pub mod owned_partition;
use crate::durability::checkpoints::aspect_state_images::CheckpointAspectContractCatalog;
use crate::durability::checkpoints::images::{
    partition_from_image_for_request, partition_to_image_with_contracts,
};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane;
use worth_relational_storage::internal_port::storage::overlay::partition::PartitionState;
use worth_relational_storage::internal_port::storage::overlay::partition_content::PartitionContentCommitment;
use worth_relational_storage::internal_port::storage::partition::adjacency_direction::AdjacencyKindBasis;
use worth_relational_storage::internal_port::storage::residency::cell::{
    RelationalColdRegion, ResidentRegion,
};
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;
use worth_relational_storage::internal_port::storage::residency::region_image::*;
use worth_relational_storage::internal_port::storage::residency::region_load::materialization_funding as funding;
use worth_relational_storage::internal_port::storage::residency::resident_history::projection_funding;

pub static CHECKPOINT_REGION_READMISSION: worth_relational_storage::internal_port::storage::residency::cold_readmission::ColdRegionReadmission =
    worth_relational_storage::internal_port::storage::residency::cold_readmission::ColdRegionReadmission::new(
        load_head,
        load_full,
        super::region_capture::capture_region_members,
        owned_partition::with_codec,
    );

fn load_head(
    cold: &RelationalColdRegion,
    source: &Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    ReconstructionLane::enter_reconstruction(|lane| {
        let (_contract_memory, contract_bound) =
            funding::contracts(cold.schema.as_ref(), execution)?;
        check_member_limit(cold.head.length, limit).map_err(|cause| unavailable(cold, cause))?;
        let _read = funding::Funding::reserve(
            funding::vector::<u8>(
                usize::try_from(cold.head.length).map_err(|_| funding::overflow())?,
            )?,
            execution,
        )?;
        funding::work(execution, cold.head.length)?;
        let bytes = source
            .read_member(cold.head, limit)
            .map_err(|cause| unavailable(cold, cause))?;
        let frame = RelationalRegionImage::decode(
            lane,
            &mut |_, _| Ok(()),
            &bytes,
            cold.head,
            cold.partition_id,
            cold.schema_carrier_digest,
            RelationalRegionImageKind::Head,
        )
        .map_err(|cause| unavailable(cold, cause))?;
        let quote =
            funding::region_image::head(funding::Raw(frame.payload), contract_bound, execution)
                .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
        // Capacity quotes reserve backing; decoding pays for the bytes it visits.
        funding::work(execution, frame.payload.len() as u64)?;
        let memory = funding::MaterializationFunding::admit(quote, execution)?;
        let (mut partition, frontiers, mut historical) = (|| -> Result<_, Failure> {
            let head: RegionHeadImage = rmp_serde::from_slice(frame.payload)
                .map_err(|_| unavailable(cold, Cause::Damaged))?;
            worth_relational_storage::internal_port::storage::residency::kind_commitment::verify(
                lane,
                &head.kind_projection,
                cold.kind_authority.head,
                cold.partition_id,
                execution,
            )?;
            worth_relational_storage::internal_port::storage::residency::kind_commitment::verify(
                lane,
                &head.history_kind_projection,
                cold.kind_authority.history,
                cold.partition_id,
                execution,
            )?;
            if head.partition.partition_id != cold.partition_id {
                return Err(unavailable(cold, Cause::Damaged).into());
            }
            if head.partition.entity_arena.metadata_history.len()
                != head.partition.entity_arena.generations.len()
                || head.partition.relation_arena.metadata_history.len()
                    != head.partition.relation_arena.generations.len()
            {
                return Err(unavailable(cold, Cause::Damaged).into());
            }
            let contracts = contracts(cold).map_err(|cause| unavailable(cold, cause))?;
            let mut partition = partition_from_image_for_request(
                lane,
                head.partition,
                cold.schema.aspect_plans(),
                &contracts,
                execution,
            )
            .map_err(|cause| owned_partition::image_failure(cause, cold.partition_id))?;
            worth_relational_storage::internal_port::storage::residency::kind_projection::reconstruction::request::rebuild_for_request(
                lane,
                &mut partition,
                &head.kind_projection,
                AdjacencyKindBasis::Current,
                execution,
            )?;
            head_validation::verify_buckets(
                &partition,
                &head.adjacency_kinds,
                &head.reverse_adjacency_kinds,
                execution,
            )
            .map_err(|cause| funding::at_region(cause, cold.partition_id))?;
            worth_relational_storage::internal_port::storage::residency::kind_projection::reconstruction::request::rebuild_head_history_for_request(
                lane,
                &mut partition,
                &head.history_kind_projection,
                execution,
            )?;
            let mut historical = BTreeMap::<usize, Vec<_>>::new();
            for claim in head.history_kind_projection {
                funding::work(execution, 1 + 11 * (historical.len() + 1).ilog2() as u64)?;
                if claim.relation.partition_id == cold.partition_id {
                    historical
                        .entry(claim.relation.slot_index())
                        .or_default()
                        .push(claim);
                }
            }
            Ok((partition, head.history_frontiers, historical))
        })()?;
        let requirements =
            worth_relational_storage::internal_port::storage::residency::required_symbols::CapturedSymbolRequirements::capture_for_request(
                lane, &partition, false, execution,
            )?;
        cold.symbols.demand_symbol_ids(
            execution,
            requirements.get(false).unwrap().ids.iter().copied(),
        )?;
        let symbols = cold.symbols.interner_snapshot();
        let commitment = PartitionContentCommitment::capture_head(
            lane, execution, &partition, &symbols, &frontiers,
        )
        .map_err(|cause| funding::content(cause, cold.partition_id))?;
        verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
        worth_relational_storage::internal_port::storage::residency::record_directory::RegionRecordDirectory::verify_member(lane, execution, &cold.directory, &partition)?;
        worth_relational_branch::internal_port::history::data::required_output_positions::presence::admission::validate(
            execution,
            &partition,
            cold.schema.registry(),
        )?;
        let history_custody = funding::history_custody::capture(cold, source, execution)?;
        let (entity_openings, relation_openings) = frontiers.into_arenas();
        install_history_prefixes(
            &mut partition.entity_arena,
            cold,
            &history_custody,
            entity_openings,
            &mut BTreeMap::new(),
            execution,
        )?;
        install_history_prefixes(
            &mut partition.relation_arena,
            cold,
            &history_custody,
            relation_openings,
            &mut historical,
            execution,
        )?;
        worth_relational_contracts::internal_port::execution::request_custody::check_request(
            execution,
        )
        .map_err(Failure::Request)?;
        Ok(Arc::new(ResidentRegion {
            symbol_requirements: OnceLock::from(Arc::new(requirements)),
            history_presence: worth_relational_storage::internal_port::storage::residency::history_presence::HistoryPresence::from_cold(lane, &partition),
            kind_indexes: Some(worth_relational_storage::internal_port::storage::residency::kind_admission::reconstruct(
                lane, execution, &partition,
            )?),
            partition,
            sealed_watermarks: worth_relational_storage::internal_port::storage::substrate::record_arena::arena::extra_allocations::SealedAllocationWatermarks::capture(),
            commitment: Some(commitment),
            _history_custody: Some(history_custody),
            _memory: memory.retained.into_memory(),
        }))
    })
}

fn load_full(
    cold: &RelationalColdRegion,
    source: &Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>,
    head: &ResidentRegion,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    limit: u64,
) -> Result<Arc<ResidentRegion>, Failure> {
    ReconstructionLane::enter_reconstruction(|lane| {
        let (_contract_memory, _) = funding::contracts(cold.schema.as_ref(), execution)?;
        let (_projection_memory, _) = projection_funding(&head.partition, execution)?;
        let retained = cold.schema.retained_aspect_contracts();
        let current_quote = funding::current(&head.partition, retained, execution)?;
        let _current_wire = funding::Funding::reserve(current_quote.wire, execution)?;
        let reference = RegionHistoryBase {
            member: cold.history,
            partition_id: cold.partition_id,
            schema_carrier_digest: cold.schema_carrier_digest,
            row_index: 0,
            kind_commitment: *cold.kind_authority.history.bytes(),
        };
        let mut folder =
            worth_relational_storage::internal_port::storage::residency::history_fold::HistoryFolder::new(lane, execution, cold.partition_id, limit);
        let history = folder.read(lane, source.as_ref(), &cold.history_members, &reference)?;
        worth_relational_storage::internal_port::storage::residency::kind_commitment::verify(
            lane,
            &history.image.kind_projection,
            cold.kind_authority.history,
            cold.partition_id,
            execution,
        )?;
        if history.image.entities.len() != head.partition.entity_arena.slot_count()
            || history.image.relations.len() != head.partition.relation_arena.slot_count()
        {
            return Err(unavailable(cold, Cause::Damaged).into());
        }
        let mut entities = folder.fold_rows::<worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::EntityRecordKind>(
            lane,
            source.as_ref(),
            &cold.history_members,
            &reference,
            history.image.entities.len(),
        )?;
        let mut relations = folder.fold_rows::<worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::RelationRecordKind>(
            lane,
            source.as_ref(),
            &cold.history_members,
            &reference,
            history.image.relations.len(),
        )?;
        drop(folder);
        let (partition, retained) = (|| -> Result<_, Failure> {
            let contracts = contracts(cold).map_err(|cause| unavailable(cold, cause))?;
            let mut image = partition_to_image_with_contracts(
                lane,
                head.partition.clone(),
                cold.schema.aspect_plans(),
                &contracts,
            )
            .map_err(|_| unavailable(cold, Cause::Damaged))?;
            for (current, earlier) in image
                .entity_arena
                .metadata_history
                .iter_mut()
                .zip(&mut entities.rows)
            {
                earlier.append_current(current, execution)?;
                *current = std::mem::take(&mut earlier.values);
            }
            for (current, earlier) in image
                .relation_arena
                .metadata_history
                .iter_mut()
                .zip(&mut relations.rows)
            {
                earlier.append_current(current, execution)?;
                *current = std::mem::take(&mut earlier.values);
            }
            let retained =
                funding::Funding::reserve(funding::shared::<ResidentRegion>()?, execution)?;
            let mut partition = partition_from_image_for_request(
                lane,
                image,
                cold.schema.aspect_plans(),
                &contracts,
                execution,
            )
            .map_err(|cause| owned_partition::image_failure(cause, cold.partition_id))?;
            partition.adjacency = head.partition.adjacency.clone();
            partition.reverse_adjacency = head.partition.reverse_adjacency.clone();
            worth_relational_storage::internal_port::storage::residency::kind_projection::reconstruction::request::rebuild_for_request(
                lane,
                &mut partition,
                &history.image.kind_projection,
                AdjacencyKindBasis::Historical,
                execution,
            )?;
            worth_relational_storage::internal_port::storage::residency::kind_admission::recover(
                lane,
                execution,
                cold.partition_id,
                |checkpoint, _| {
                    verify_buckets(
                        lane,
                        &partition,
                        &history.image.adjacency_kinds,
                        &history.image.reverse_adjacency_kinds,
                        AdjacencyKindBasis::Historical,
                        checkpoint,
                    )
                },
            )?;
            Ok((partition, retained))
        })()?;
        let requirements =
            worth_relational_storage::internal_port::storage::residency::required_symbols::CapturedSymbolRequirements::capture_for_request(
                lane, &partition, true, execution,
            )?;
        cold.symbols.demand_symbol_ids(
            execution,
            requirements.get(true).unwrap().ids.iter().copied(),
        )?;
        let symbols = cold.symbols.interner_snapshot();
        let (commitment, _) =
            PartitionContentCommitment::capture(lane, execution, &partition, &symbols)
                .map_err(|cause| funding::content(cause, cold.partition_id))?;
        verify_leaf(cold, &partition, &commitment).map_err(|cause| unavailable(cold, cause))?;
        worth_relational_storage::internal_port::storage::residency::record_directory::RegionRecordDirectory::verify_member(lane, execution, &cold.directory, &partition)?;
        worth_relational_contracts::internal_port::execution::request_custody::check_request(
            execution,
        )
        .map_err(Failure::Request)?;
        Ok(Arc::new(ResidentRegion {
            symbol_requirements: OnceLock::from(Arc::new(requirements)),
            history_presence: worth_relational_storage::internal_port::storage::residency::history_presence::HistoryPresence::complete(),
            kind_indexes: Some(worth_relational_storage::internal_port::storage::residency::kind_admission::reconstruct(
                lane, execution, &partition,
            )?),
            partition,
            sealed_watermarks: worth_relational_storage::internal_port::storage::substrate::record_arena::arena::extra_allocations::SealedAllocationWatermarks::capture(),
            commitment: Some(commitment),
            _history_custody: None,
            _memory: retained.into_memory(),
        }))
    })
}

pub mod admission;
use admission::*;
