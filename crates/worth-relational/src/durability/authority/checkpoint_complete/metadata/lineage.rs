//! Full original canonical inputs supply published nodes, events and digest totals.
use super::super::source::OriginalCanonicalCommit;
use super::{array_stop, map_stop, overflow};
use crate::durability::data::{
    CheckpointLineageBasis, RelationalNativeCheckpointCaptureDenial as Denial,
    RelationalRecoveryStop,
};
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::history::data::CommitId;
use crate::identity::data::{EntityId, LineageId};
use crate::lineage::data::{LineageEventKind, LineageEventRecord, LineageNode};
use crate::publication::patch::data::RecordStructuralChange;
use crate::transactions::data::RecordRef;
use serde::Serialize;
use worth_execution::{
    ExecutionArray, ExecutionImmutableBytes, ExecutionOrderedMap, ExecutionRequest,
    ExecutionRequestArrayBuilder,
};

#[derive(Clone, Copy, Serialize)]
#[serde(rename = "RelationalCheckpointLineageEventRow")]
pub(in crate::durability::authority::checkpoint_complete) struct OriginalLineageEvent<'a> {
    pub(in crate::durability::authority::checkpoint_complete) event: &'a LineageEventRecord,
    pub(in crate::durability::authority::checkpoint_complete) publication_commit_id: CommitId,
}
pub(in crate::durability::authority::checkpoint_complete) struct OriginalCheckpointLineage<'a> {
    pub(in crate::durability::authority::checkpoint_complete) basis: CheckpointLineageBasis,
    pub(in crate::durability::authority::checkpoint_complete) nodes: ExecutionArray<LineageNode>,
    pub(in crate::durability::authority::checkpoint_complete) events:
        ExecutionArray<OriginalLineageEvent<'a>>,
}
pub(in crate::durability::authority::checkpoint_complete) fn capture_lineage<'a>(
    commits: &'a [OriginalCanonicalCommit],
    prior: Option<&crate::durability::data::RelationalCheckpointManifest>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OriginalCheckpointLineage<'a>, Denial> {
    let mut nodes = ExecutionOrderedMap::<LineageId, EntityId>::default();
    let mut events = ExecutionOrderedMap::<u64, OriginalLineageEvent<'a>>::default();
    let prior_basis = prior.map(|prior| &prior.lineage_basis);
    let (mut commit_count, mut event_count, mut decision_count, mut id_count) =
        prior_basis.map_or((0, 0, 0, 0), |basis| {
            (
                basis.published_lineage_commit_count,
                basis.published_lineage_event_count,
                basis.published_lineage_decision_count,
                basis.canonical_published_event_ids.len(),
            )
        });
    let mut covered_nodes = ExecutionOrderedMap::<LineageId, ()>::default();
    if let Some(prior) = prior {
        for page in &prior.lineage {
            for id in page.first..=page.last {
                if page.contains(id) {
                    covered_nodes
                        .insert_cloned_native_for_request(&LineageId(id), &(), *request)
                        .map_err(map_stop)?;
                }
            }
        }
    }
    for original in commits {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        let envelope = &original.envelope;
        let basis = envelope.lineage_digest_basis();
        commit_count = commit_count
            .checked_add(usize::from(envelope.has_lineage_authority()))
            .ok_or_else(overflow)?;
        event_count = event_count
            .checked_add(basis.lineage_event_count())
            .ok_or_else(overflow)?;
        decision_count = decision_count
            .checked_add(basis.lineage_decision_count())
            .ok_or_else(overflow)?;
        id_count = id_count
            .checked_add(basis.canonical_event_ids().len())
            .ok_or_else(overflow)?;
        let mut creates = envelope
            .lineage_events()
            .iter()
            .filter(|e| e.kind() == LineageEventKind::Create);
        for patch in &envelope.patch.authoritative_record_patches {
            let RecordRef::Entity(entity) = patch.target else {
                continue;
            };
            if patch.structural_change != RecordStructuralChange::Created {
                continue;
            }
            let event = creates.next().ok_or(Denial::OriginalAuthorityUnavailable)?;
            if event.commit() != &envelope.commit
                || event.targets().len() != 1
                || !event.sources().is_empty()
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let lineage = event.targets()[0];
            if covered_nodes.contains_key(&lineage) {
                continue;
            }
            if let Some(previous) = nodes.get(&lineage) {
                if previous != &entity {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            } else {
                nodes
                    .insert_cloned_native_for_request(&lineage, &entity, *request)
                    .map_err(map_stop)?;
            }
        }
        if creates.next().is_some() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        for event in envelope.lineage_events() {
            let row = OriginalLineageEvent {
                event,
                publication_commit_id: envelope.commit.commit_id,
            };
            if let Some(previous) = events.get(&event.event_id()) {
                if previous.event != row.event
                    || previous.publication_commit_id != row.publication_commit_id
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            } else {
                events
                    .insert_cloned_native_for_request(&event.event_id(), &row, *request)
                    .map_err(map_stop)?;
            }
        }
    }
    let basis = CheckpointLineageBasis::from_original(
        commit_count,
        event_count,
        decision_count,
        id_count,
        prior_basis
            .into_iter()
            .flat_map(|basis| basis.canonical_published_event_ids.iter().copied())
            .chain(commits.iter().flat_map(|c| {
                c.envelope
                    .lineage_digest_basis()
                    .canonical_event_ids()
                    .iter()
                    .copied()
            })),
        request,
    )
    .map_err(Denial::Request)?;
    let mut node_roster =
        ExecutionRequestArrayBuilder::allocate(*request, nodes.len()).map_err(array_stop)?;
    for (&lineage_id, &entity_id) in nodes.iter() {
        node_roster
            .push(LineageNode {
                lineage_id,
                entity_id,
            })
            .map_err(array_stop)?;
    }
    let mut event_roster =
        ExecutionRequestArrayBuilder::allocate(*request, events.len()).map_err(array_stop)?;
    for (_, event) in events.iter() {
        event_roster.push(*event).map_err(array_stop)?;
    }
    Ok(OriginalCheckpointLineage {
        basis,
        nodes: node_roster.seal().map_err(array_stop)?,
        events: event_roster.seal().map_err(array_stop)?,
    })
}
pub(in crate::durability::authority::checkpoint_complete) fn encode_lineage(
    nodes: &[LineageNode],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalDurableMemberLocator, ExecutionImmutableBytes), RelationalRecoveryStop> {
    member_frame::encode_for_request(MemberKind::Lineage, &nodes, request)
}
pub(in crate::durability::authority::checkpoint_complete) fn encode_lineage_events(
    rows: &[OriginalLineageEvent<'_>],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(RelationalDurableMemberLocator, ExecutionImmutableBytes), RelationalRecoveryStop> {
    member_frame::encode_for_request(MemberKind::LineageEvents, &rows, request)
}
