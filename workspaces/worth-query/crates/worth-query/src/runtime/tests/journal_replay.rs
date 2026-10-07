use super::support::*;
use crate::application::WorthQueryJournalReplaySurfaceEvidence;
use crate::evidence_identity::WorthQueryEvidenceIdentityScheme;
use std::collections::BTreeSet;

#[test]
fn journal_segment_replay_returns_receipts_schedule_and_artifact_digest() {
    let mut workspace = replay_workspace("tasks.journal-replay.success");
    let first = submit_task(&mut workspace, "task-1", "First");
    let second = submit_task(&mut workspace, "task-2", "Second");
    let third = submit_task(&mut workspace, "task-3", "Third");
    let segment = WorthQueryJournalSegmentIdentity::between(
        first.journal_position(),
        third.journal_position(),
    )
    .expect("committed segment should build");
    let _published_generation = workspace
        .shared_read_context()
        .expect("shared read context should mint a published generation");

    let outcome = workspace
        .replay_journal_segment(WorthQueryJournalReplayRequest::new(segment))
        .expect("segment replay should succeed");

    assert_eq!(
        outcome.write_receipts(),
        &[first.clone(), second.clone(), third.clone()]
    );
    assert_eq!(
        outcome.position_schedule().positions(),
        outcome
            .write_receipts()
            .iter()
            .map(WorthQueryWriteReceipt::journal_position)
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(outcome.position_schedule().expected_position_count(), 3);
    assert_eq!(
        outcome
            .position_schedule()
            .stable_replay_count(outcome.position_schedule()),
        3
    );
    let replay_evidence = WorthQueryJournalReplaySurfaceEvidence::derive_from_committed_receipts(
        &outcome,
        3,
        &[first.clone(), second.clone(), third.clone()],
        workspace
            .journal_replay_diagnostics()
            .counter_snapshot()
            .clone(),
    );
    assert_eq!(
        outcome.truth_reconstruction_identity().as_str(),
        replay_evidence.replay_truth_digest()
    );
    assert!(replay_evidence.certified());
    assert_eq!(outcome.expected_journal_position_count(), 3);
    assert_eq!(outcome.resolved_journal_position_count(), 3);
    assert_eq!(outcome.journal_gap_count(), 0);
    assert_eq!(outcome.scanned_entry_count(), 3);
    let counter_snapshot = replay_evidence.counter_snapshot();
    assert_eq!(counter_snapshot.retained_entry_count(), 3);
    assert_eq!(counter_snapshot.replay_admission_count(), 1);
    assert_eq!(counter_snapshot.replay_scanned_entry_count(), 3);
    assert_eq!(counter_snapshot.replay_resolved_entry_count(), 3);
    assert_eq!(counter_snapshot.replay_gap_count(), 0);
    assert_eq!(counter_snapshot.replay_residue_count(), 0);
    assert_eq!(
        counter_snapshot.last_replay_outcome_digest(),
        Some(outcome.outcome_digest())
    );
    assert!(
        workspace
            .runtime
            .published_artifact_diagnostics()
            .retained_generation_count()
            > 0
    );
    assert!(!outcome.published_artifact_digest().as_str().is_empty());
    assert!(!outcome.outcome_digest().is_empty());
}

#[test]
fn journal_replay_stale_basis_denial_is_typed_and_leaves_replay_available() {
    let mut workspace = replay_workspace("tasks.journal-replay.stale-basis");
    let first = submit_task(&mut workspace, "task-1", "First");
    let stale_basis = workspace
        .snapshot_identity()
        .expect("the fixture owner remains open");
    submit_task(&mut workspace, "task-2", "Second");
    let segment = WorthQueryJournalSegmentIdentity::between(
        first.journal_position(),
        first.journal_position(),
    )
    .expect("single-position segment should build");

    let error = workspace
        .replay_journal_segment(
            WorthQueryJournalReplayRequest::new(segment.clone()).with_basis_snapshot(stale_basis),
        )
        .expect_err("stale basis should deny replay");

    assert_journal_replay_denial(error, WorthQueryJournalReplayDenialKind::StaleBasisReplay);
    assert_residue_free_denial(
        &workspace,
        WorthQueryJournalReplayDenialKind::StaleBasisReplay,
        2,
        1,
    );
    assert!(workspace
        .replay_journal_segment(WorthQueryJournalReplayRequest::new(segment))
        .is_ok());
}

#[test]
fn journal_replay_unknown_and_gap_segments_fail_with_typed_denials() {
    let mut workspace = replay_workspace("tasks.journal-replay.unknown-gap");
    let first = submit_task(&mut workspace, "task-1", "First");
    let _second = submit_task(&mut workspace, "task-2", "Second");
    let third = submit_task(&mut workspace, "task-3", "Third");
    let segment = WorthQueryJournalSegmentIdentity::between(
        first.journal_position(),
        third.journal_position(),
    )
    .expect("committed segment should build");

    workspace.retain_journal_replay_positions_for_certification(&BTreeSet::new());
    let unknown_error = workspace
        .replay_journal_segment(WorthQueryJournalReplayRequest::new(segment.clone()))
        .expect_err("retained-away segment should deny as unknown");
    assert_journal_replay_denial(
        unknown_error,
        WorthQueryJournalReplayDenialKind::UnknownSegmentIdentity,
    );
    assert_residue_free_denial(
        &workspace,
        WorthQueryJournalReplayDenialKind::UnknownSegmentIdentity,
        0,
        1,
    );

    let cross_scheme =
        WorthQueryJournalSegmentIdentity::admit_versioned_committed_bounds_for_replay(
            1,
            1,
            WorthQueryEvidenceIdentityScheme::V2,
        )
        .expect("versioned structured segment should build");
    let cross_scheme_error = workspace
        .replay_journal_segment(WorthQueryJournalReplayRequest::new(cross_scheme))
        .expect_err("cross-scheme segment should deny replay");
    assert_journal_replay_denial(
        cross_scheme_error,
        WorthQueryJournalReplayDenialKind::CrossSchemeReplay,
    );
    assert_residue_free_denial(
        &workspace,
        WorthQueryJournalReplayDenialKind::CrossSchemeReplay,
        0,
        1,
    );

    let mut gap_workspace = replay_workspace("tasks.journal-replay.gap");
    let gap_first = submit_task(&mut gap_workspace, "task-1", "First");
    let gap_second = submit_task(&mut gap_workspace, "task-2", "Second");
    let gap_third = submit_task(&mut gap_workspace, "task-3", "Third");
    let gap = WorthQueryJournalSegmentIdentity::between(
        gap_first.journal_position(),
        gap_third.journal_position(),
    )
    .expect("committed gap segment should build");
    let retained_positions = BTreeSet::from([
        gap_first.journal_position().ordinal_for_reporting(),
        gap_third.journal_position().ordinal_for_reporting(),
    ]);
    gap_workspace.retain_journal_replay_positions_for_certification(&retained_positions);
    let gap_error = gap_workspace
        .replay_journal_segment(WorthQueryJournalReplayRequest::new(gap))
        .expect_err("gap segment should deny replay");
    assert_journal_replay_denial(gap_error, WorthQueryJournalReplayDenialKind::JournalGap);
    assert_residue_free_denial(
        &gap_workspace,
        WorthQueryJournalReplayDenialKind::JournalGap,
        2,
        1,
    );
    assert_eq!(gap_second.journal_position().ordinal_for_reporting(), 2);
}

#[test]
fn journal_replay_surface_is_admitted_by_runtime_support_profile() {
    let workspace = replay_workspace("tasks.journal-replay.support-profile");
    let contract = workspace.public_api_contract();
    let row = contract
        .family(WorthQueryRuntimeFacadeFamily::Replay)
        .expect("replay facade support row should exist");

    assert_eq!(
        row.status(),
        WorthQueryRuntimeFamilySupportStatus::Supported
    );
    assert!(row.ordinary_downstream_dx());
    assert!(row
        .evidence()
        .iter()
        .any(|label| label == "journal-replay-outcome"));
}

fn assert_residue_free_denial(
    workspace: &WorthQueryWorkspace,
    kind: WorthQueryJournalReplayDenialKind,
    retained_entry_count: usize,
    denial_count: usize,
) {
    let snapshot = workspace
        .journal_replay_diagnostics()
        .counter_snapshot()
        .clone();
    assert_eq!(snapshot.retained_entry_count(), retained_entry_count);
    assert_eq!(snapshot.denial_count(kind), denial_count);
    assert_eq!(snapshot.replay_residue_count(), 0);
    assert_eq!(snapshot.replay_admission_count(), 0);
    assert_eq!(snapshot.last_replay_outcome_digest(), None);
}

fn replay_workspace(name: &str) -> WorthQueryWorkspace {
    stateful_bridge_task_runtime()
        .workspace(name)
        .expect("task runtime should open replay workspace")
}

fn submit_task(
    workspace: &mut WorthQueryWorkspace,
    id: &str,
    title: &str,
) -> WorthQueryWriteReceipt {
    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("identity.id"),
            test_authored_string_aspect_value(id),
        )
        .set_aspect(
            test_aspect_touch("title.value"),
            test_authored_string_aspect_value(title),
        )
        .build_insert("Task")
        .expect("insert command should build");
    workspace
        .submissions()
        .expect("submission lane should be admitted")
        .submit(command)
        .expect("submission should commit")
}

fn assert_journal_replay_denial(
    error: WorthQueryRuntimeError,
    expected: WorthQueryJournalReplayDenialKind,
) {
    match error {
        WorthQueryRuntimeError::JournalReplayDenied(denial) => assert_eq!(denial.kind(), expected),
        other => panic!("expected journal replay denial, got {other:?}"),
    }
}
