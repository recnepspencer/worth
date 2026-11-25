//! Recovered lineage artifacts must agree with their canonical basis, never
//! reuse an event id, and allocate every `Create` and `Restore` lineage once,
//! in ascending order within each commit.
use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::{
    CanonicalCommitEnvelope, CanonicalCommitEnvelopeBodyOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;

use crate::durability::authority::lineage_allocation::{
    allocated_lineage, maximum_allocated_lineage,
};
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::identity::data::primitives::LineageId;
use contracts::lineage::data::events::LineageEventRecord;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::LineageSubsystem;

pub fn validate_recovered_lineage_artifacts<'a>(
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
    let relation_lineages = envelope.lineage_relation_lineages();
    if !relation_lineages.is_empty() {
        let relation_event_targets: BTreeSet<LineageId> = envelope
            .lineage_events()
            .iter()
            .filter(|event| {
                event.subject() == contracts::lineage::data::events::LineageSubject::Relation
                    && matches!(
                        event.kind(),
                        contracts::lineage::data::events::LineageEventKind::Create
                            | contracts::lineage::data::events::LineageEventKind::Restore
                    )
            })
            .flat_map(|event| event.targets().iter().copied())
            .collect();
        let mut seen_relations = BTreeSet::new();
        let mut seen_lineages = BTreeSet::new();
        for &(relation_id, lineage_id) in relation_lineages {
            if !seen_relations.insert(relation_id)
                || !seen_lineages.insert(lineage_id)
                || !relation_event_targets.contains(&lineage_id)
            {
                return Err(DurabilityError::new(
                    failure_class,
                    format!(
                        "recovered relation lineage binding ({relation_id:?}, {lineage_id:?}) for commit {} conflicts with its lineage events",
                        envelope.commit.commit_id.0
                    ),
                ));
            }
        }
    }
    for event in envelope.lineage_events() {
        if let Some(rel_id) = event.relation_id() {
            let valid_binding = event.subject()
                == contracts::lineage::data::events::LineageSubject::Relation
                && matches!(
                    event.kind(),
                    contracts::lineage::data::events::LineageEventKind::Create
                        | contracts::lineage::data::events::LineageEventKind::Restore
                )
                && event.targets().len() == 1
                && relation_lineages.contains(&(rel_id, event.targets()[0]));
            if !valid_binding {
                return Err(DurabilityError::new(
                    failure_class,
                    format!(
                        "recovered lineage event {} relation binding ({rel_id:?}) conflicts with commit {} relation_lineages",
                        event.event_id(),
                        envelope.commit.commit_id.0
                    ),
                ));
            }
        }
    }
    // An explicit entity binding must anchor to an entity this commit
    // rematerialized, on an entity `Create`/`Restore`, at most once.
    let view = crate::durability::authority::lineage_allocation::CommitAllocationView::of_envelope(
        envelope,
    );
    crate::durability::authority::lineage_allocation::explicit_entity_bindings(&view).map_err(
        |refusal| {
            DurabilityError::new(
                failure_class.clone(),
                refusal.describe(envelope.commit.commit_id.0),
            )
        },
    )?;
    let observed = contracts::lineage::data::artifacts::finalization_artifact::LineageFinalizationArtifact::new(
        envelope.branch_context.clone(),
        contracts::lineage::data::artifacts::event_batch::FinalizedLineageEventBatch::new(envelope.lineage_events().to_vec()),
        contracts::lineage::data::artifacts::decision_log::LineageDecisionLog::new(envelope.lineage_decision_log().to_vec()),
    )
    .with_relation_lineages(relation_lineages.to_vec())
    .with_group_lineage(envelope.published_lineage().group_lineage().to_vec())
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

pub fn reconcile_recovered_lineage_artifacts(
    lane: &contracts::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    lineage: &LineageSubsystem,
    tail_log: &[PositionedCanonicalCommit],
) -> Result<(), DurabilityError> {
    let mut events_by_id = BTreeMap::new();
    for (event, publication_commit_id) in lineage.drain_events() {
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

    for positioned in tail_log {
        let envelope = positioned.envelope();
        let durable = super::recovered_group_lineage::recovered_group_lineage(
            [envelope],
            RecoveryFailureClass::ReplayFailure,
        )?;
        let durable = durable.into_iter().next().map(|(_, _, t)| t);
        let replayed =
            lineage.published_group_lineage_at(&envelope.branch_context, envelope.commit.commit_id);
        if replayed != durable.unwrap_or_default() {
            return Err(replay_failure(format!(
                "replayed group lineage of commit {} disagrees with its durable canonical frame",
                envelope.commit.commit_id.0
            )));
        }
    }

    let next_event_id = events_by_id
        .last_key_value()
        .and_then(|(event_id, _)| event_id.checked_add(1));
    let next_lineage_id = lineage
        .maximum_node_id()
        .max(maximum_allocated_lineage(
            events_by_id.values().map(|(event, _)| event),
        ))
        .and_then(|lineage_id| lineage_id.checked_add(1));
    lineage.replace_events(lane, events_by_id.into_values().collect());
    lineage.advance_identity_to(next_lineage_id, next_event_id);
    Ok(())
}

fn replay_failure(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail)
}
