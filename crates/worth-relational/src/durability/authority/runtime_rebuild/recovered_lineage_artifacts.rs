//! Recovered lineage artifacts must agree with their canonical basis, never
//! reuse an event id, and allocate every `Create` and `Restore` lineage once,
//! in ascending order within each commit.
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use super::super::lineage_allocation::allocated_lineage;
use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::history::data::{
    BranchId, CanonicalCommitEnvelope, PositionedCanonicalCommit, RelationalCommitReceipt,
};
use crate::identity::data::LineageId;
use crate::lineage::data::LineageEventRecord;
use crate::runtime::RelationalRuntime;

pub(super) fn validate_recovered_lineage_artifacts<'a>(
    checkpoint: Option<&DurableCheckpoint>,
    tail_log: impl IntoIterator<Item = &'a CanonicalCommitEnvelope>,
) -> Result<(), DurabilityError> {
    let mut ledger = RecoveredLineageIdentityLedger::default();
    if let Some(checkpoint) = checkpoint {
        for envelope in &checkpoint.envelopes {
            validate_envelope_lineage(
                envelope.envelope(),
                RecoveryFailureClass::CorruptCheckpoint,
                &mut ledger,
            )?;
        }
    }
    for envelope in tail_log {
        validate_envelope_lineage(
            envelope,
            RecoveryFailureClass::CorruptCanonicalArtifact,
            &mut ledger,
        )?;
    }
    Ok(())
}

fn validate_envelope_lineage(
    envelope: &CanonicalCommitEnvelope,
    failure_class: RecoveryFailureClass,
    ledger: &mut RecoveredLineageIdentityLedger,
) -> Result<(), DurabilityError> {
    let observed = crate::lineage::data::LineageFinalizationArtifact::new(
        envelope.branch_context.clone(),
        crate::lineage::data::FinalizedLineageEventBatch::new(envelope.lineage_events().to_vec()),
        crate::lineage::data::LineageDecisionLog::new(envelope.lineage_decision_log().to_vec()),
    )
    .publish();
    if &observed != envelope.published_lineage() {
        return Err(DurabilityError::new(
            failure_class,
            format!(
                "recovered lineage artifact for commit {} disagrees with its observed canonical basis",
                envelope.commit.commit_id.0
            ),
        ));
    }
    ledger.admit_commit_events(
        &envelope.commit,
        &envelope.branch_context,
        envelope.lineage_events(),
        failure_class,
    )
}

/// Event ids and allocated lineage ids already admitted by recovery.
#[derive(Default)]
pub(super) struct RecoveredLineageIdentityLedger {
    seen_event_ids: BTreeSet<u64>,
    seen_allocated_lineage_ids: BTreeSet<LineageId>,
}

impl RecoveredLineageIdentityLedger {
    /// Admits one commit's events: each belongs to its commit and branch,
    /// event ids are fresh and ascending, and every `Create` or `Restore`
    /// lineage of either subject is fresh and ascending within the commit.
    ///
    /// Complexity: O(events log admitted).
    pub(super) fn admit_commit_events(
        &mut self,
        commit: &RelationalCommitReceipt,
        branch: &BranchId,
        events: &[LineageEventRecord],
        failure_class: RecoveryFailureClass,
    ) -> Result<(), DurabilityError> {
        let refuse = |detail: String| Err(DurabilityError::new(failure_class.clone(), detail));
        let mut previous_event_id = 0;
        let mut previous_allocated = LineageId(0);
        for event in events {
            if event.commit() != commit || event.branch_id() != branch {
                return refuse(format!(
                    "recovered lineage event {} is cross-spliced from its containing commit envelope {}",
                    event.event_id(),
                    commit.commit_id.0
                ));
            }
            if !self.seen_event_ids.insert(event.event_id()) {
                return refuse(format!(
                    "recovered lineage event id {} is reused by multiple canonical artifacts",
                    event.event_id()
                ));
            }
            if event.event_id() <= previous_event_id {
                return refuse(format!(
                    "recovered lineage event id {} does not advance within its canonical batch after {}",
                    event.event_id(),
                    previous_event_id
                ));
            }
            previous_event_id = event.event_id();
            let allocation = match allocated_lineage(event) {
                Ok(allocation) => allocation,
                Err(refusal) => return refuse(refusal.describe(commit.commit_id.0)),
            };
            let Some(allocation) = allocation else {
                continue;
            };
            let lineage_id = allocation.lineage;
            if !self.seen_allocated_lineage_ids.insert(lineage_id) {
                return refuse(format!(
                    "recovered allocated lineage id {} is reused by multiple canonical artifacts",
                    lineage_id.0
                ));
            }
            if lineage_id.0 <= previous_allocated.0 {
                return refuse(format!(
                    "recovered allocated lineage id {} does not advance after {}",
                    lineage_id.0, previous_allocated.0
                ));
            }
            previous_allocated = lineage_id;
        }
        Ok(())
    }
}

pub(super) fn reconcile_recovered_lineage_artifacts(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    restored: &mut RelationalRuntime,
    tail_log: &[PositionedCanonicalCommit],
) -> Result<(), DurabilityError> {
    let mut events_by_id = BTreeMap::new();
    for (event, publication_commit_id) in restored.lineage.drain_events() {
        match events_by_id.entry(event.event_id()) {
            Entry::Vacant(entry) => {
                entry.insert((event, publication_commit_id));
            }
            Entry::Occupied(_) => {
                return Err(replay_failure(format!(
                    "replay reconstructed duplicate lineage event id {}",
                    event.event_id()
                )));
            }
        }
    }

    for envelope in tail_log {
        for durable_event in envelope.lineage_events() {
            let publication_commit_id = envelope.commit.commit_id;
            match events_by_id.entry(durable_event.event_id()) {
                Entry::Occupied(entry)
                    if entry.get() == &(durable_event.clone(), publication_commit_id) => {}
                Entry::Occupied(_) => {
                    return Err(replay_failure(format!(
                        "replayed lineage event {} conflicts with its durable canonical artifact",
                        durable_event.event_id()
                    )));
                }
                Entry::Vacant(_) => {
                    return Err(replay_failure(format!(
                        "replay omitted durable lineage event {} for commit {}",
                        durable_event.event_id(),
                        envelope.commit.commit_id.0
                    )));
                }
            }
        }
    }

    let next_event_id = events_by_id
        .last_key_value()
        .and_then(|(event_id, _)| event_id.checked_add(1));
    let next_lineage_id = restored
        .lineage
        .maximum_node_id()
        .and_then(|lineage_id| lineage_id.checked_add(1));
    restored
        .lineage
        .replace_events(lane, events_by_id.into_values().collect());
    restored
        .lineage
        .advance_identity_to(next_lineage_id, next_event_id);
    Ok(())
}

fn replay_failure(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail)
}
