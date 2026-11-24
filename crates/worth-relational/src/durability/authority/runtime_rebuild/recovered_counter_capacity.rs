//! Recovery lineage allocator frontiers advance across every lineage-allocating
//! `Create` and `Restore` event, so replay never reuses a recorded lineage id.
use super::super::lineage_allocation::{allocated_lineage, maximum_allocated_lineage};
use crate::durability::data::{DurabilityError, RecoveryFailureClass};
use crate::runtime::state::subsystems::{
    durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    history::HistorySubsystem, lineage::LineageSubsystem,
};

pub(super) fn prepare_recovery_lineage_sequence(
    lineage: &LineageSubsystem,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) {
    if let Some(first_event_id) = envelope
        .lineage_events()
        .first()
        .map(|event| event.event_id())
    {
        let next_lineage_id = lineage.identity_frontiers().0;
        lineage.set_identity_frontiers(next_lineage_id, first_event_id);
    }
    if let Some(first_allocated_lineage_id) = first_allocated_lineage(envelope.lineage_events()) {
        let next_event_id = lineage.identity_frontiers().1;
        lineage.set_identity_frontiers(first_allocated_lineage_id.0, next_event_id);
    }
}

pub(super) fn validate_tail_lineage_allocator_capacity<'a>(
    tail_log: impl IntoIterator<Item = &'a crate::history::data::CanonicalCommitEnvelope>,
) -> Result<(), DurabilityError> {
    let envelopes = tail_log.into_iter().collect::<Vec<_>>();
    if envelopes
        .iter()
        .flat_map(|commit| commit.lineage_events())
        .any(|event| event.event_id() >= u64::MAX - 1)
    {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            "recovery tail lineage event id exhausted the allocator",
        ));
    }
    if allocations_exhaust_allocator(envelopes.iter().flat_map(|commit| commit.lineage_events())) {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            "recovery tail lineage id exhausted the allocator",
        ));
    }
    Ok(())
}

/// The first lineage a commit allocates, in event order: replay must
/// reallocate exactly it, whether a `Create` or a `Restore` allocated it.
pub(super) fn first_allocated_lineage(
    events: &[crate::lineage::data::LineageEventRecord],
) -> Option<crate::identity::data::LineageId> {
    events
        .iter()
        .find_map(|event| allocated_lineage(event).ok().flatten())
        .map(|allocation| allocation.lineage)
}

/// Whether any `Create` or `Restore` allocation leaves the lineage-id
/// allocator no safe successor.
pub(super) fn allocations_exhaust_allocator<'a>(
    events: impl IntoIterator<Item = &'a crate::lineage::data::LineageEventRecord>,
) -> bool {
    events
        .into_iter()
        .filter_map(|event| allocated_lineage(event).ok().flatten())
        .any(|allocation| allocation.lineage.0 >= u64::MAX - 1)
}

pub(super) fn refresh_checkpoint_counters(
    history: &HistorySubsystem,
    lineage: &LineageSubsystem,
    checkpoint: &RelationalCheckpointManifestSelection,
) -> Result<(), DurabilityError> {
    let recorded_envelopes = history.commit_artifacts();
    let next_commit_id = checked_checkpoint_successor(
        recorded_envelopes
            .iter()
            .map(|artifact| artifact.commit_id().0)
            .max(),
        "commit id",
    )?;
    let next_version_id = checked_checkpoint_successor(
        recorded_envelopes
            .iter()
            .map(|artifact| artifact.version_id().0)
            .max(),
        "version id",
    )?;
    let directory = checkpoint.snapshot();
    let cold_node_max = directory
        .as_ref()
        .and_then(|directory| directory.manifest.lineage.last().map(|page| page.last));
    let cold_event_max = directory.as_ref().and_then(|directory| {
        directory
            .manifest
            .lineage_basis
            .canonical_published_event_ids
            .iter()
            .max()
            .copied()
    });
    let cold_event_lineage_max = directory.as_ref().and_then(|directory| {
        directory
            .manifest
            .lineage_events
            .iter()
            .map(|page| page.last_node)
            .max()
    });
    let resident_events = lineage.shared_events();
    let next_lineage_id = recovered_lineage_frontier(
        lineage
            .maximum_node_id()
            .max(cold_node_max)
            .max(cold_event_lineage_max),
        resident_events.iter().map(std::sync::Arc::as_ref),
    )?;
    let next_event_id = checked_checkpoint_successor(
        lineage.maximum_event_id().max(cold_event_max),
        "lineage event id",
    )?;
    history.install_recovered_sequence_floor(next_commit_id, next_version_id);
    lineage.set_identity_frontiers(next_lineage_id, next_event_id);
    Ok(())
}

/// The recovered lineage-id frontier: strictly past every recorded entity
/// node and cold event page (`recorded_maximum`) and past every resident
/// `Create` or `Restore` allocation of either subject, refusing exhaustion.
pub(super) fn recovered_lineage_frontier<'event>(
    recorded_maximum: Option<u64>,
    resident_events: impl IntoIterator<Item = &'event crate::lineage::data::LineageEventRecord>,
) -> Result<u64, DurabilityError> {
    checked_checkpoint_successor(
        recorded_maximum.max(maximum_allocated_lineage(resident_events)),
        "lineage id",
    )
}

fn checked_checkpoint_successor(
    maximum: Option<u64>,
    counter_name: &str,
) -> Result<u64, DurabilityError> {
    match maximum {
        Some(maximum) if maximum >= u64::MAX - 1 => Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("recovered checkpoint {counter_name} has no safe next transition"),
        )),
        Some(maximum) => Ok(maximum + 1),
        None => Ok(1),
    }
}
