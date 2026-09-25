use rayon::prelude::*;

use crate::authority::commit::preparation::packets::import::{
    ImportFragmentIdentity, ImportFragmentKind, ImportStagedRow, ImportStagingHeader,
    ImportStagingPacket,
};
use crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection;
use crate::authority::commit::preparation::planning::strategy::{
    coarse_preparation_packet_count, strategy_for_parallel_packets,
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
use crate::authority::mutation::record_changes::allocate_relation;
use crate::authority::mutation::MutationWorkspace;
use crate::transactions::data::{
    AspectFieldPatch, BulkImportRowDomain, BulkImportStage, BulkRelationCreateIntent,
    CommitConflict, ConflictClass, CreatedRelationRef, EntityReference, RecordAspectPatchTarget,
};
use crate::validation::data::InvariantGroupSet;
use worth_foundational::facade::PortablePatchReadmissionPurpose;

use super::field_authoring_candidate::{self, FieldAuthoringDomain};
use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    intent: &BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let version_id = workspace.version_id();
    let mut outcome = MutationOutcome::with_capacity(intent.endpoints.len(), 1);
    outcome.record_event(
        crate::authority::mutation::outcomes::MutationEvent::BulkRelationsCreated {
            partition_id: intent.partition_id,
            kind_id: intent.kind_id,
            count: 0,
        },
    );
    for_each_staged_bulk_relation_row(intent, workspace, &mut outcome, version_id)?;
    outcome.set_last_event_count(intent.endpoints.len());
    Ok(outcome)
}

fn for_each_staged_bulk_relation_row(
    intent: &BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    outcome: &mut MutationOutcome,
    version_id: crate::identity::data::VersionId,
) -> Result<(), CommitConflict> {
    if intent.endpoints.len() <= TARGET_PREPARATION_ITEMS_PER_PACKET {
        workspace.record_preparation_strategy(
            1,
            intent.endpoints.len(),
            intent.endpoints.len(),
            1,
            strategy_for_parallel_packets(workspace.execution_model(), 1),
        );
        for (offset, (source, target)) in intent.endpoints.iter().cloned().enumerate() {
            let fields = intent
                .field_patches
                .get(offset)
                .cloned()
                .unwrap_or_default();
            apply_staged_relation_row(
                intent,
                workspace,
                outcome,
                version_id,
                intent.client_keys.get(offset).cloned(),
                source,
                target,
                fields,
            )?;
        }
        return Ok(());
    }

    let packet_count = coarse_preparation_packet_count(
        intent.endpoints.len(),
        TARGET_PREPARATION_ITEMS_PER_PACKET,
    );
    let mut packets = Vec::with_capacity(packet_count);
    for (packet_index, endpoints) in intent
        .endpoints
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
                    fragment_kind: ImportFragmentKind::RelationCreate,
                    packet_index: packet_index_floor,
                },
                proof_kind: PreparationProofKind::FragmentIdentityDisjoint,
                locality: PreparationLocalityProof {
                    observation_scope:
                        crate::validation::engine::InvariantObservationKind::Speculative,
                    record_domain: PreparationRecordDomain::Relation,
                    partition_scope: PreparationPartitionScope::TouchedPartitions(
                        vec![intent.partition_id].into(),
                    ),
                    invariant_group_scope: InvariantGroupSet::empty(),
                    read_set_approximation: PreparationReadSetApproximation::TouchedOnly,
                    write_exclusion: PreparationWriteExclusionClass::PublicationExcluded,
                },
            },
            rows: endpoints
                .iter()
                .cloned()
                .enumerate()
                .map(|(offset, (source, target))| ImportStagedRow::Relation {
                    client_key: intent.client_keys.get(packet_index_floor + offset).cloned(),
                    source,
                    target,
                    fields: intent
                        .field_patches
                        .get(packet_index_floor + offset)
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect(),
        });
    }

    for row in stage_import_packets(workspace, packets) {
        match row {
            ImportStagedRow::Relation {
                client_key,
                source,
                target,
                fields,
            } => apply_staged_relation_row(
                intent, workspace, outcome, version_id, client_key, source, target, fields,
            )?,
            ImportStagedRow::Entity { .. } => {
                return Err(CommitConflict::new(
                    crate::transactions::data::ConflictClass::BulkImportDomainMismatch {
                        expected: BulkImportRowDomain::Relation,
                        actual: BulkImportRowDomain::Entity,
                        stage: BulkImportStage::RelationCreate,
                    },
                ))
            }
        }
    }
    Ok(())
}

fn apply_staged_relation_row(
    intent: &BulkRelationCreateIntent,
    workspace: &mut MutationWorkspace<'_>,
    outcome: &mut MutationOutcome,
    version_id: crate::identity::data::VersionId,
    client_key: Option<crate::symbols::data::ClientKey>,
    source: EntityReference,
    target: EntityReference,
    fields: AspectFieldPatch,
) -> Result<(), CommitConflict> {
    let source_id = resolve_entity_reference(workspace, &source)?;
    let target_id = resolve_entity_reference(workspace, &target)?;
    let patch_target = RecordAspectPatchTarget::RelationCreation {
        kind_id: intent.kind_id,
    };
    let plan = workspace.relation_aspect_plan(intent.kind_id);
    let candidate = field_authoring_candidate::lower(
        &fields,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        intent.kind_id,
        FieldAuthoringDomain::Relation,
    )
    .map_err(|denial| record_aspect_patch::conflict(patch_target, denial))?;
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        candidate, plan, source_id, target_id,
    );
    let authoritative_patch = record_aspect_patch::readmit(
        candidate,
        PortablePatchReadmissionPurpose::RecordCreation,
        plan,
        patch_target,
    )?;
    let authoritative_aspect_state =
        record_aspect_patch::apply(None, &authoritative_patch, patch_target)?;
    let relation_id = workspace.with_context(|context| {
        let relation_id = allocate_relation(
            context.state,
            context.record_allocations,
            version_id,
            intent.partition_id,
            intent.kind_id,
            source_id,
            target_id,
            authoritative_aspect_state,
        )?;
        context
            .state
            .mark_relation_slot_touched(relation_id.partition_id, relation_id.slot_index());
        Ok(relation_id)
    })?;
    if let Some(client_key) = client_key {
        workspace.register_created_relation(
            CreatedRelationRef {
                partition_id: intent.partition_id,
                kind_id: intent.kind_id,
                client_key,
                source: source.clone(),
                target: target.clone(),
            },
            relation_id,
        );
    }
    outcome.record_change(RecordMutation::RelationCreated {
        relation_id,
        kind_id: intent.kind_id,
        source: source_id,
        target: target_id,
        authoritative_patch: record_aspect_patch::published_patch(authoritative_patch),
    });
    Ok(())
}

fn resolve_entity_reference(
    workspace: &MutationWorkspace<'_>,
    entity_reference: &EntityReference,
) -> Result<crate::identity::data::EntityId, CommitConflict> {
    workspace
        .resolve_entity_reference(entity_reference)
        .ok_or_else(|| {
            CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
                detail:
                    "relation endpoints must resolve within the same authoritative commit scope"
                        .to_string(),
            })
        })
}

fn stage_import_packets(
    workspace: &mut MutationWorkspace<'_>,
    packets: Vec<ImportStagingPacket>,
) -> Vec<ImportStagedRow> {
    if packets.is_empty() {
        return Vec::new();
    }

    let strategy = strategy_for_parallel_packets(workspace.execution_model(), packets.len());
    let packet_item_count = packets.iter().map(|packet| packet.rows.len()).sum();
    let packet_max_width = packets
        .iter()
        .map(|packet| packet.rows.len())
        .max()
        .unwrap_or(0);
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
        // Packets are emitted in canonical packet-index order and relation rows are already
        // ordered within each packet, so the serial path can flatten directly.
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

fn import_packet_stream(
    packet: &ImportStagingPacket,
) -> OrderedReductionStream<ImportReductionKey, ImportStagedRow> {
    let mut stream = Vec::with_capacity(packet.rows.len());
    for (offset, row) in packet.rows.iter().cloned().enumerate() {
        stream.push((
            ImportReductionKey::new(
                packet.header.identity.partition_id,
                1,
                packet.header.packet_index_floor + offset,
            ),
            row,
        ));
    }
    OrderedReductionStream::new(stream)
}
