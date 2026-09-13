use worth_foundational::canonicalization_api::lower_lane::basis::CanonicalBasisEntry;

use crate::{
    CheckpointCrashReplayObservation, CheckpointInterlockObservation,
    CompactionInterlockObservation, ObservedPhysicalTrace,
};

use super::super::ExecutedTranscriptParts;
use super::entries::text_entry;

pub(super) fn observation_entries(parts: &ExecutedTranscriptParts) -> Vec<CanonicalBasisEntry> {
    let trace = parts.trace();
    let mut entries = generic_trace_observation_entries(trace);
    entries.extend(checkpoint_crash_replay_entries(
        trace.checkpoint_crash_replay(),
    ));
    entries.extend(checkpoint_interlock_entries(trace.checkpoint_interlock()));
    entries.extend(compaction_interlock_entries(trace.compaction_interlock()));
    entries
}

fn generic_trace_observation_entries(trace: &ObservedPhysicalTrace) -> Vec<CanonicalBasisEntry> {
    let mut entries = vec![
        text_entry(
            "transcript.trace.observation_basis",
            format!("{:?}", trace.observation_basis()),
        ),
        text_entry(
            "transcript.trace.independent_verifier",
            trace
                .independent_verifier()
                .map(|observation| format!("{:?}:{:?}", observation.seam(), observation.kind()))
                .unwrap_or_else(|| "none".to_owned()),
        ),
        text_entry(
            "transcript.trace.recovery_outcome",
            trace
                .recovery_outcome()
                .map(|observation| format!("{:?}", observation.kind()))
                .unwrap_or_else(|| "none".to_owned()),
        ),
    ];
    entries.extend(
        trace
            .shortcut_rejections()
            .iter()
            .enumerate()
            .map(|(index, observation)| {
                text_entry(
                    format!("transcript.trace.shortcut_rejection.{index:04}"),
                    format!("{:?}", observation.kind()),
                )
            }),
    );
    entries
}

fn checkpoint_crash_replay_entries(
    observation: Option<&CheckpointCrashReplayObservation>,
) -> Vec<CanonicalBasisEntry> {
    let Some(observation) = observation else {
        return vec![text_entry(
            "transcript.trace.checkpoint_crash_replay",
            "none",
        )];
    };
    vec![
        text_entry("transcript.trace.checkpoint_crash_replay", "present"),
        text_entry(
            "transcript.trace.checkpoint_crash_replay.checkpoint_origin",
            format!("{:?}", observation.checkpoint_origin()),
        ),
        text_entry(
            "transcript.trace.checkpoint_crash_replay.recovery_outcome",
            format!("{:?}", observation.recovery_outcome().kind()),
        ),
        count_entry(
            "transcript.trace.checkpoint_crash_replay.checkpoint_actor_step",
            observation.checkpoint_actor_step_index() as u64,
        ),
        count_entry(
            "transcript.trace.checkpoint_crash_replay.recovery_actor_step",
            observation.recovery_actor_step_index() as u64,
        ),
        text_entry(
            "transcript.trace.checkpoint_crash_replay.recovery_plan",
            format!("{:?}", observation.recovery_plan_identity()),
        ),
        text_entry(
            "transcript.trace.checkpoint_crash_replay.recovery_schedule",
            format!("{:?}", observation.recovery_schedule_identity()),
        ),
    ]
}

fn checkpoint_interlock_entries(
    observation: Option<CheckpointInterlockObservation>,
) -> Vec<CanonicalBasisEntry> {
    let Some(observation) = observation else {
        return vec![text_entry("transcript.trace.checkpoint_interlock", "none")];
    };
    vec![
        text_entry("transcript.trace.checkpoint_interlock", "present"),
        bool_entry(
            "transcript.trace.checkpoint_interlock.no_mixed_root",
            observation.no_mixed_root(),
        ),
        bool_entry(
            "transcript.trace.checkpoint_interlock.old_reader_retained_old_root",
            observation.old_reader_retained_old_root(),
        ),
        bool_entry(
            "transcript.trace.checkpoint_interlock.post_reader_new_epoch",
            observation.post_publication_reader_observed_new_epoch(),
        ),
        bool_entry(
            "transcript.trace.checkpoint_interlock.frontier_bound_to_cutover",
            observation.checkpoint_wal_bound_to_cutover(),
        ),
        count_entry(
            "transcript.trace.checkpoint_interlock.root_epoch_checks",
            observation.root_epoch_checks(),
        ),
        count_entry(
            "transcript.trace.checkpoint_interlock.manifest_epoch_checks",
            observation.manifest_epoch_checks(),
        ),
        count_entry(
            "transcript.trace.checkpoint_interlock.frontier_checks",
            observation.checkpoint_wal_range_checks(),
        ),
        count_entry(
            "transcript.trace.checkpoint_interlock.readmission_checks",
            observation.readmission_checks(),
        ),
        count_entry(
            "transcript.trace.checkpoint_interlock.publication_swaps",
            observation.publication_swaps(),
        ),
    ]
}

fn compaction_interlock_entries(
    observation: Option<CompactionInterlockObservation>,
) -> Vec<CanonicalBasisEntry> {
    let Some(observation) = observation else {
        return vec![text_entry("transcript.trace.compaction_interlock", "none")];
    };
    vec![
        text_entry("transcript.trace.compaction_interlock", "present"),
        bool_entry(
            "transcript.trace.compaction_interlock.no_mixed_root",
            observation.no_mixed_root(),
        ),
        bool_entry(
            "transcript.trace.compaction_interlock.old_reader_old_structure",
            observation.old_reader_retained_old_structure(),
        ),
        bool_entry(
            "transcript.trace.compaction_interlock.new_reader_new_epoch",
            observation.new_reader_observed_new_epoch(),
        ),
        bool_entry(
            "transcript.trace.compaction_interlock.blocked_reclaim",
            observation.blocked_reclaim_until_release(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.protected_ranges",
            observation.protected_ranges(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.candidate_ranges",
            observation.candidate_ranges(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.range_comparisons",
            observation.range_comparisons(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.overlapping_ranges",
            observation.overlapping_ranges(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.copied_pages",
            observation.copied_pages(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.publication_swaps",
            observation.publication_swaps(),
        ),
        count_entry(
            "transcript.trace.compaction_interlock.blocked_reclaims",
            observation.blocked_reclaims(),
        ),
    ]
}

fn bool_entry(locus: &'static str, value: bool) -> CanonicalBasisEntry {
    text_entry(locus, value.to_string())
}

fn count_entry(locus: &'static str, value: u64) -> CanonicalBasisEntry {
    text_entry(locus, value.to_string())
}
