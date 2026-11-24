use crate::durability::authority::lineage_allocation::{
    commit_allocations, entity_allocation_bindings, CommitAllocationView,
};
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::identity::data::primitives::LineageId;
use contracts::identity::data::record_ids::EntityId;
use contracts::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointDigestBasis;
use std::collections::{BTreeMap, BTreeSet};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn validate_checkpoint_lineage_artifact(
    checkpoint: &DurableCheckpoint,
) -> Result<(), DurabilityError> {
    validate_published_lineage_artifacts(checkpoint)?;
    validate_checkpoint_lineage_nodes(checkpoint)?;
    let published_lineage_commit_count = checkpoint
        .envelopes
        .iter()
        .filter(|envelope| envelope.has_lineage_authority())
        .count();
    let canonical_published_event_ids = checkpoint
        .envelopes
        .iter()
        .flat_map(|envelope| {
            envelope
                .lineage_digest_basis()
                .canonical_event_ids()
                .iter()
                .copied()
        })
        .collect();
    let published_lineage_event_count = checkpoint
        .envelopes
        .iter()
        .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
        .sum();
    let published_lineage_decision_count = checkpoint
        .envelopes
        .iter()
        .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
        .sum();
    let observed_basis = LineageCheckpointDigestBasis::new(
        published_lineage_commit_count,
        canonical_published_event_ids,
        published_lineage_event_count,
        published_lineage_decision_count,
    );
    if checkpoint.lineage.digest_basis() != &observed_basis {
        return Err(DurabilityError::new(
            RecoveryFailureClass::ReplayFailure,
            "durable checkpoint lineage artifact basis drifted from canonical published lineage",
        ));
    }
    Ok(())
}

fn validate_published_lineage_artifacts(
    checkpoint: &DurableCheckpoint,
) -> Result<(), DurabilityError> {
    for envelope in &checkpoint.envelopes {
        let expected = contracts::lineage::data::artifacts::finalization_artifact::LineageFinalizationArtifact::new(
            envelope.branch_context.clone(),
            contracts::lineage::data::artifacts::event_batch::FinalizedLineageEventBatch::new(
                envelope.lineage_events().to_vec(),
            ),
            contracts::lineage::data::artifacts::decision_log::LineageDecisionLog::new(envelope.lineage_decision_log().to_vec()),
        )
        .with_relation_lineages(envelope.lineage_relation_lineages().to_vec())
        .publish();
        if &expected != envelope.published_lineage() {
            return Err(corrupt_lineage(format!(
                "checkpoint commit {} carries lineage events or decisions that disagree with their observed canonical basis",
                envelope.commit.commit_id.0
            )));
        }
    }
    Ok(())
}

fn validate_checkpoint_lineage_nodes(
    checkpoint: &DurableCheckpoint,
) -> Result<(), DurabilityError> {
    let counters = checkpoint.lineage.counters();
    if counters.node_count != checkpoint.lineage.nodes().len() {
        return Err(corrupt_lineage(
            "checkpoint lineage counters disagree with their observed artifacts".to_owned(),
        ));
    }
    let nodes = checkpoint
        .lineage
        .nodes()
        .iter()
        .map(|node| (node.lineage_id(), node.entity_id()))
        .collect::<BTreeMap<_, _>>();
    if nodes.len() != checkpoint.lineage.nodes().len() {
        return Err(corrupt_lineage(
            "checkpoint lineage nodes reuse a lineage identity".to_owned(),
        ));
    }

    validate_entity_allocation_nodes(
        &nodes,
        checkpoint
            .envelopes
            .iter()
            .map(|positioned| CommitAllocationView::of_envelope(positioned.envelope())),
    )
}

/// Every lineage allocated across `commits` is unique, and the canonical
/// nodes are exactly the entity allocations, each naming the entity its
/// commit created.
///
/// Complexity: O(total events + total created entities log created).
pub(super) fn validate_entity_allocation_nodes<'commit>(
    nodes: &BTreeMap<LineageId, EntityId>,
    commits: impl IntoIterator<Item = CommitAllocationView<'commit>>,
) -> Result<(), DurabilityError> {
    let mut allocated_lineages = BTreeSet::new();
    let mut entity_lineages = BTreeSet::new();
    for view in commits {
        let commit_id = view.commit.commit_id.0;
        let allocations = commit_allocations(view.events)
            .map_err(|refusal| corrupt_lineage(refusal.describe(commit_id)))?;
        for allocation in allocations {
            if !allocated_lineages.insert(allocation.lineage) {
                return Err(corrupt_lineage(format!(
                    "checkpoint allocated lineage id {} is reused",
                    allocation.lineage.0
                )));
            }
        }
        let bindings = entity_allocation_bindings(&view)
            .map_err(|refusal| corrupt_lineage(refusal.describe(commit_id)))?;
        for (lineage_id, expected_entity_id) in bindings {
            let Some(entity_id) = nodes.get(&lineage_id) else {
                return Err(corrupt_lineage(format!(
                    "checkpoint entity lineage id {} has no canonical node",
                    lineage_id.0
                )));
            };
            if entity_id != &expected_entity_id {
                return Err(corrupt_lineage(format!(
                    "checkpoint lineage node {} does not name its exact entity in commit {commit_id}",
                    lineage_id.0,
                )));
            }
            entity_lineages.insert(lineage_id);
        }
    }
    if nodes.keys().copied().collect::<BTreeSet<_>>() != entity_lineages {
        return Err(corrupt_lineage(
            "checkpoint lineage nodes are not bijective with canonical entity allocations"
                .to_owned(),
        ));
    }
    Ok(())
}

fn corrupt_lineage(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
