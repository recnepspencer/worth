use rayon::prelude::*;

use crate::authority::commit::preparation::packets::import::{
    ImportFragmentIdentity, ImportFragmentKind, ImportStagedRow, ImportStagingHeader,
    ImportStagingPacket,
};
use crate::authority::commit::preparation::planning::strategy::{
    coarse_preparation_packet_count, strategy_for_parallel_packets, PreparationStrategySelection,
    TARGET_PREPARATION_ITEMS_PER_PACKET,
};
use crate::authority::commit::preparation::proofs::kinds::PreparationProofKind;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationLocalityProof, PreparationPartitionScope, PreparationReadSetApproximation,
    PreparationRecordDomain, PreparationWriteExclusionClass,
};
use crate::authority::commit::preparation::reduction::keys::ImportReductionKey;
use crate::authority::commit::preparation::reduction::merge::{
    canonical_merge_streams, OrderedReductionStream,
};
use crate::authority::mutation::outcomes::{MutationOutcome, RecordMutation};
use crate::authority::mutation::record_changes::allocate_entity_with_extra;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    BulkEntityCreateIntent, BulkImportRowDomain, BulkImportStage, CommitConflict, CreatedEntityRef,
    RecordAspectPatchTarget,
};
use crate::validation::data::InvariantGroupSet;
use worth_foundational::facade::{
    AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState, PortablePatchReadmissionPurpose,
};

use super::field_authoring_candidate::FieldAuthoringDomain;
use super::record_aspect_patch;

pub(super) fn apply(
    intent: &BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let version_id = workspace.version_id();
    let mut outcome = MutationOutcome::with_capacity(intent.field_patches.len(), 1);
    outcome.record_event(
        crate::authority::mutation::outcomes::MutationEvent::BulkEntitiesCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    );
    let staged_rows = stage_bulk_entity_rows(intent, workspace)?;
    let entity_aspect_plans = stage_bulk_entity_aspect_plans(intent, workspace, &staged_rows)?;
    for ((client_key, _fields), aspect_plan) in intent
        .client_keys
        .iter()
        .cloned()
        .zip(staged_rows.into_iter())
        .zip(entity_aspect_plans.into_iter())
    {
        let entity_id = workspace.with_context(|context| {
            let entity_id = allocate_entity_with_extra(
                context.state,
                context.record_allocations,
                version_id,
                intent.partition_id,
                intent.kind_id,
                crate::storage::substrate::EntityExtra {
                    authoritative_aspect_state: aspect_plan.1,
                    ..crate::storage::substrate::EntityExtra::default()
                },
            )?;
            context
                .state
                .mark_entity_slot_touched(entity_id.partition_id, entity_id.slot_index());
            Ok(entity_id)
        })?;
        workspace.register_created_entity(
            CreatedEntityRef {
                partition_id: intent.partition_id,
                kind_id: intent.kind_id,
                client_key,
            },
            entity_id,
        );
        outcome.record_change(RecordMutation::EntityCreated {
            entity_id,
            kind_id: intent.kind_id,
            authoritative_patch: record_aspect_patch::published_patch(aspect_plan.0),
        });
    }
    outcome.set_last_event_count(intent.field_patches.len());
    Ok(outcome)
}

fn stage_bulk_entity_aspect_plans(
    intent: &BulkEntityCreateIntent,
    workspace: &MutationWorkspace<'_>,
    field_patches: &[crate::transactions::data::AspectFieldPatch],
) -> Result<
    Vec<(
        AuthoritativeRecordAspectPatch,
        Option<AuthoritativeRecordAspectState>,
    )>,
    CommitConflict,
> {
    let lowered_plan = workspace.entity_aspect_plan(intent.kind_id);
    let target = RecordAspectPatchTarget::EntityCreation {
        kind_id: intent.kind_id,
    };
    field_patches
        .iter()
        .map(|fields| {
            let patch = record_aspect_patch::readmit_field_authoring(
                fields,
                PortablePatchReadmissionPurpose::RecordCreation,
                lowered_plan,
                target,
                FieldAuthoringDomain::Entity,
            )?;
            let state = record_aspect_patch::apply(None, &patch, target)?;
            Ok((patch, state))
        })
        .collect()
}

fn stage_bulk_entity_rows(
    intent: &BulkEntityCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<Vec<crate::transactions::data::AspectFieldPatch>, CommitConflict> {
    let packet_count = coarse_preparation_packet_count(
        intent.field_patches.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    let mut packets = Vec::with_capacity(packet_count);
    for (packet_index, field_patches) in intent
        .field_patches
        .chunks(TARGET_PREPARATION_ITEMS_PER_PACKET)
        .enumerate()
    {
        let packet_index_floor = packet_index * TARGET_PREPARATION_ITEMS_PER_PACKET;
        packets.push(ImportStagingPacket {
            header: ImportStagingHeader {
                packet_index_floor,
                identity: ImportFragmentIdentity {
                    partition_id: intent.partition_id,
                    kind_id: intent.kind_id,
                    fragment_kind: ImportFragmentKind::EntityCreate,
                    packet_index: packet_index_floor,
                },
                proof_kind: PreparationProofKind::FragmentIdentityDisjoint,
                locality: PreparationLocalityProof {
                    observation_scope:
                        crate::validation::engine::InvariantObservationKind::Speculative,
                    record_domain: PreparationRecordDomain::Entity,
                    partition_scope: PreparationPartitionScope::TouchedPartitions(
                        vec![intent.partition_id].into(),
                    ),
                    invariant_group_scope: InvariantGroupSet::empty(),
                    read_set_approximation: PreparationReadSetApproximation::TouchedOnly,
                    write_exclusion: PreparationWriteExclusionClass::PublicationExcluded,
                },
            },
            rows: field_patches
                .iter()
                .cloned()
                .map(|fields| ImportStagedRow::Entity { fields })
                .collect(),
        });
    }

    stage_import_packets(workspace, packets)
        .into_iter()
        .map(|row| match row {
            ImportStagedRow::Entity { fields } => Ok(fields),
            ImportStagedRow::Relation { .. } => Err(CommitConflict::new(
                crate::transactions::data::ConflictClass::BulkImportDomainMismatch {
                    expected: BulkImportRowDomain::Entity,
                    actual: BulkImportRowDomain::Relation,
                    stage: BulkImportStage::EntityCreate,
                },
            )),
        })
        .collect()
}

fn stage_import_packets(
    workspace: &mut MutationWorkspace<'_>,
    packets: Vec<ImportStagingPacket>,
) -> Vec<ImportStagedRow> {
    if packets.is_empty() {
        return Vec::new();
    }

    let packet_item_count = packets.iter().map(|packet| packet.rows.len()).sum();
    let packet_max_width = packets
        .iter()
        .map(|packet| packet.rows.len())
        .max()
        .unwrap_or(0);
    let strategy = record_import_strategy(workspace, packets.len());
    workspace.record_preparation_strategy(
        packets.len(),
        packet_item_count,
        packet_max_width,
        1,
        strategy,
    );
    if packets.len() == 1 {
        return packets.into_iter().next().unwrap().rows;
    }
    match strategy.selected_mode {
        // Packets are constructed in canonical packet-index order and each packet row is already
        // locally ordered, so serial execution can flatten directly without a redundant k-way
        // merge.
        PreparationStrategySelection::Serial => {
            packets.into_iter().flat_map(|packet| packet.rows).collect()
        }
        PreparationStrategySelection::StagedParallel => canonical_merge_streams(
            packets
                .par_iter()
                .map(import_packet_stream)
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|(_, row)| row)
        .collect(),
    }
}

fn record_import_strategy(
    workspace: &MutationWorkspace<'_>,
    packet_count: usize,
) -> crate::authority::commit::preparation::planning::strategy::PreparationStrategy {
    strategy_for_parallel_packets(workspace.execution_model(), packet_count)
}

fn import_packet_stream(
    packet: &ImportStagingPacket,
) -> OrderedReductionStream<ImportReductionKey, ImportStagedRow> {
    let mut stream = Vec::with_capacity(packet.rows.len());
    for (offset, row) in packet.rows.iter().cloned().enumerate() {
        stream.push((
            ImportReductionKey::new(
                packet.header.identity.partition_id,
                0,
                packet.header.packet_index_floor + offset,
            ),
            row,
        ));
    }
    OrderedReductionStream::new(stream)
}
