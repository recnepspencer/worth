use super::support::*;
use crate::application::{
    WorthQueryJournalIdentityBoundaryPosture, WorthQueryJournalIdentityCertification,
    WorthQueryJournalReplaySurfaceEvidence,
};

pub(super) fn journal_replay_surface_evidence() -> WorthQueryJournalReplaySurfaceEvidence {
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("tasks.journal-position.certification-replay")
        .expect("task runtime should open replay workspace");
    let first = submit_task(&mut workspace, "task-1", "First");
    let second = submit_task(&mut workspace, "task-2", "Second");
    let segment = WorthQueryJournalSegmentIdentity::between(
        first.journal_position(),
        second.journal_position(),
    )
    .expect("committed segment identity should build");
    let request = WorthQueryJournalReplayRequest::new(segment).with_basis_snapshot(
        workspace
            .snapshot_identity()
            .expect("the fixture owner remains open"),
    );
    let outcome = workspace
        .replay_journal_segment(request)
        .expect("journal replay should succeed");
    let counter_snapshot = workspace
        .journal_replay_diagnostics()
        .counter_snapshot()
        .clone();

    WorthQueryJournalReplaySurfaceEvidence::derive_from_committed_receipts(
        &outcome,
        2,
        &[first, second],
        counter_snapshot,
    )
}

pub(super) fn assert_closed_replay_boundary_certification(
    certification: &WorthQueryJournalIdentityCertification,
) {
    let replay_boundary = certification.replay_boundary_certification();
    assert!(!replay_boundary.journal_segment_identity_digest().is_empty());
    assert!(!replay_boundary.journal_replay_outcome_digest().is_empty());
    assert!(!replay_boundary.journal_replay_truth_digest().is_empty());
    assert!(!replay_boundary
        .published_artifact_replay_digest()
        .is_empty());
    assert!(!replay_boundary
        .journal_identity_inventory_digest()
        .is_empty());
    assert_eq!(
        replay_boundary.journal_boundary_posture(),
        WorthQueryJournalIdentityBoundaryPosture::Closed
    );
    assert_eq!(
        replay_boundary.failure_digest(),
        certification.failure_digest()
    );
    assert_eq!(replay_boundary.counter_snapshot().replay_residue_count(), 0);
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
        .expect("replay certification command should commit")
}
