use std::collections::{btree_map::Entry, BTreeMap, BTreeSet};

use crate::durability::data::{DurabilityError, DurableCheckpoint, RecoveryFailureClass};
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::runtime::RelationalRuntime;

pub(super) fn validate_recovered_lineage_artifacts<'a>(
    checkpoint: Option<&DurableCheckpoint>,
    tail_log: impl IntoIterator<Item = &'a CanonicalCommitEnvelope>,
) -> Result<(), DurabilityError> {
    let mut seen_event_ids = BTreeSet::new();
    let mut seen_created_lineage_ids = BTreeSet::new();
    if let Some(checkpoint) = checkpoint {
        for envelope in &checkpoint.envelopes {
            let mut previous_event_id = Some(0);
            let mut previous_created_lineage_id = Some(crate::identity::data::LineageId(0));
            validate_envelope_lineage(
                envelope.envelope(),
                RecoveryFailureClass::CorruptCheckpoint,
                &mut seen_event_ids,
                &mut seen_created_lineage_ids,
                &mut previous_event_id,
                &mut previous_created_lineage_id,
            )?;
        }
    }
    for envelope in tail_log {
        let mut previous_event_id = Some(0);
        let mut previous_created_lineage_id = Some(crate::identity::data::LineageId(0));
        validate_envelope_lineage(
            envelope,
            RecoveryFailureClass::CorruptCanonicalArtifact,
            &mut seen_event_ids,
            &mut seen_created_lineage_ids,
            &mut previous_event_id,
            &mut previous_created_lineage_id,
        )?;
    }
    Ok(())
}

fn validate_envelope_lineage(
    envelope: &CanonicalCommitEnvelope,
    failure_class: RecoveryFailureClass,
    seen_event_ids: &mut BTreeSet<u64>,
    seen_created_lineage_ids: &mut BTreeSet<crate::identity::data::LineageId>,
    previous_event_id: &mut Option<u64>,
    previous_created_lineage_id: &mut Option<crate::identity::data::LineageId>,
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
    for event in envelope.lineage_events() {
        let commit_axes_match = event.commit() == &envelope.commit;
        if !commit_axes_match || event.branch_id() != &envelope.branch_context {
            return Err(DurabilityError::new(
                failure_class,
                format!(
                    "recovered lineage event {} is cross-spliced from its containing commit envelope {}",
                    event.event_id(),
                    envelope.commit.commit_id.0
                ),
            ));
        }
        if !seen_event_ids.insert(event.event_id()) {
            return Err(DurabilityError::new(
                failure_class,
                format!(
                    "recovered lineage event id {} is reused by multiple canonical artifacts",
                    event.event_id()
                ),
            ));
        }
        if previous_event_id.is_some_and(|previous| event.event_id() <= previous) {
            return Err(DurabilityError::new(
                failure_class,
                format!(
                    "recovered lineage event id {} does not advance within its canonical batch after {}",
                    event.event_id(),
                    previous_event_id.expect("checked predecessor exists")
                ),
            ));
        }
        if event.kind() == crate::lineage::data::LineageEventKind::Create {
            for lineage_id in event.targets() {
                if !seen_created_lineage_ids.insert(*lineage_id) {
                    return Err(DurabilityError::new(
                        failure_class,
                        format!(
                            "recovered create lineage id {} is reused by multiple canonical artifacts",
                            lineage_id.0
                        ),
                    ));
                }
                if previous_created_lineage_id.is_some_and(|previous| lineage_id.0 <= previous.0) {
                    return Err(DurabilityError::new(
                        failure_class,
                        format!(
                            "recovered create lineage id {} does not advance after {}",
                            lineage_id.0,
                            previous_created_lineage_id
                                .expect("checked predecessor exists")
                                .0
                        ),
                    ));
                }
                *previous_created_lineage_id = Some(*lineage_id);
            }
        }
        *previous_event_id = Some(event.event_id());
    }
    Ok(())
}

pub(super) fn reconcile_recovered_lineage_artifacts(
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
        .replace_events(events_by_id.into_values().collect());
    restored
        .lineage
        .advance_identity_to(next_lineage_id, next_event_id);
    Ok(())
}

fn replay_failure(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail)
}
