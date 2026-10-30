use super::super::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

fn qualified(root: &std::path::Path, schedule: MediaFaultSchedule) -> QualifiedFilesystemMedia {
    let request = FilesystemQualificationRequest::certification(
        root,
        FilesystemAccessPosture::CoordinatedServiceAccount,
    )
    .with_fault_schedule(schedule);
    match FilesystemMediaOwner::qualify(request).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("qualification failed"),
    }
}

fn artifacts(
    media: &QualifiedFilesystemMedia,
) -> (ArtifactTreeDirectory, ArtifactTreeFile, ArtifactTreeFile) {
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&records).unwrap();
    let current = records.file("bootstrap.catalog").unwrap();
    let candidate = records
        .file("bootstrap-0000000000000001.candidate")
        .unwrap();
    tree.write_new(&current, &[1; 64]).unwrap();
    tree.write_new(&candidate, &[2; 64]).unwrap();
    (records, current, candidate)
}

fn baseline_ordinal(role: MediaOperationRole) -> u64 {
    let parent = tempfile::tempdir().unwrap();
    let media = qualified(
        &parent.path().join("baseline"),
        MediaFaultSchedule::default(),
    );
    artifacts(&media);
    let ordinal = media.counters().attempts_for(role);
    media.close();
    ordinal
}

fn paused_schedule(
    role: MediaOperationRole,
    ordinal: u64,
    gate: MediaPauseGate,
) -> MediaFaultSchedule {
    MediaFaultSchedule::for_certification(vec![MediaFaultRule::for_certification(
        role,
        ordinal,
        MediaFaultDirective::PauseBefore(gate),
    )])
    .unwrap()
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: positioned write holds the path until replacement finishes after it; source: worth-store-physical-backend::filesystem_media::tests::artifact_mutation_coordination::positioned_write_holds_the_path_until_replacement_finishes_after_it; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=2, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn positioned_write_holds_the_path_until_replacement_finishes_after_it() {
    let parent = tempfile::tempdir().unwrap();
    let gate = MediaPauseGate::for_certification();
    let schedule = paused_schedule(
        MediaOperationRole::PositionedWrite,
        baseline_ordinal(MediaOperationRole::PositionedWrite) + 1,
        gate.clone(),
    );
    let media = qualified(&parent.path().join("store"), schedule);
    let (_, current, candidate) = artifacts(&media);
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 64).unwrap();

    std::thread::scope(|scope| {
        let tree = media.artifact_tree();
        let write_target = current.clone();
        let write = scope.spawn(move || tree.write_exact_at(&write_target, coordinate, &[7; 64]));
        gate.wait_until_reached();
        let replacement_tree = media.artifact_tree();
        let replacement = scope.spawn(move || replacement_tree.replace(&candidate, &current));
        media
            .artifact_tree_owner()
            .wait_until_artifact_mutation_is_contended();
        assert_eq!(
            media
                .counters()
                .attempts_for(MediaOperationRole::AtomicReplace),
            baseline_ordinal(MediaOperationRole::AtomicReplace),
            "replacement reached the OS while the exact write still owned the path"
        );
        gate.release();
        assert!(matches!(
            write.join().unwrap(),
            ArtifactRangeWriteOutcome::Completed(_)
        ));
        replacement.join().unwrap().unwrap();
    });
    assert_eq!(
        std::fs::read(
            parent
                .path()
                .join("store/families/records/bootstrap.catalog")
        )
        .unwrap(),
        [2; 64]
    );
    media.close();
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: replacement first forces the later write to open the replacement; source: worth-store-physical-backend::filesystem_media::tests::artifact_mutation_coordination::replacement_first_forces_the_later_write_to_open_the_replacement; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=2, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn replacement_first_forces_the_later_write_to_open_the_replacement() {
    let parent = tempfile::tempdir().unwrap();
    let gate = MediaPauseGate::for_certification();
    let schedule = paused_schedule(
        MediaOperationRole::AtomicReplace,
        baseline_ordinal(MediaOperationRole::AtomicReplace) + 1,
        gate.clone(),
    );
    let media = qualified(&parent.path().join("store"), schedule);
    let (_, current, candidate) = artifacts(&media);
    let coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 64).unwrap();

    std::thread::scope(|scope| {
        let replacement_tree = media.artifact_tree();
        let replacement_target = current.clone();
        let replacement =
            scope.spawn(move || replacement_tree.replace(&candidate, &replacement_target));
        gate.wait_until_reached();
        let write_tree = media.artifact_tree();
        let write = scope.spawn(move || write_tree.write_exact_at(&current, coordinate, &[9; 64]));
        media
            .artifact_tree_owner()
            .wait_until_artifact_mutation_is_contended();
        gate.release();
        replacement.join().unwrap().unwrap();
        assert!(matches!(
            write.join().unwrap(),
            ArtifactRangeWriteOutcome::Completed(_)
        ));
    });
    assert_eq!(
        std::fs::read(
            parent
                .path()
                .join("store/families/records/bootstrap.catalog")
        )
        .unwrap(),
        [9; 64]
    );
    media.close();
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: disjoint artifact writes do not share a global store lock; source: worth-store-physical-backend::filesystem_media::tests::artifact_mutation_coordination::disjoint_artifact_writes_do_not_share_a_global_store_lock; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=2, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn disjoint_artifact_writes_do_not_share_a_global_store_lock() {
    let parent = tempfile::tempdir().unwrap();
    let gate = MediaPauseGate::for_certification();
    let schedule = paused_schedule(
        MediaOperationRole::PositionedWrite,
        baseline_ordinal(MediaOperationRole::PositionedWrite) + 1,
        gate.clone(),
    );
    let media = qualified(&parent.path().join("store"), schedule);
    let (_, current, candidate) = artifacts(&media);
    let current_coordinate =
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, 64).unwrap();
    let candidate_coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::CatalogCandidate { publication: 1 },
        0,
        64,
    )
    .unwrap();

    std::thread::scope(|scope| {
        let first_tree = media.artifact_tree();
        let first =
            scope.spawn(move || first_tree.write_exact_at(&current, current_coordinate, &[3; 64]));
        gate.wait_until_reached();
        let second_tree = media.artifact_tree();
        let (sent, received) = std::sync::mpsc::sync_channel(1);
        let second = scope.spawn(move || {
            let outcome = second_tree.write_exact_at(&candidate, candidate_coordinate, &[4; 64]);
            sent.send(()).unwrap();
            outcome
        });
        let completed_while_first_paused = received
            .recv_timeout(std::time::Duration::from_secs(2))
            .is_ok();
        gate.release();
        assert!(
            completed_while_first_paused,
            "disjoint artifact was globally serialized"
        );
        assert!(matches!(
            first.join().unwrap(),
            ArtifactRangeWriteOutcome::Completed(_)
        ));
        assert!(matches!(
            second.join().unwrap(),
            ArtifactRangeWriteOutcome::Completed(_)
        ));
    });
    media.close();
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: coordinated storage-owner unit-test harness; purpose: opposing replacements reserve both paths without deadlock; source: worth-store-physical-backend::filesystem_media::tests::artifact_mutation_coordination::opposing_replacements_reserve_both_paths_without_deadlock; lexical counts: std::thread::spawn=0, std::thread::scope=1, std::thread::Scope::spawn=2, available_parallelism=0; barriers, joins and original bounds coordinate the storage-owner claim"
)]
#[test]
fn opposing_replacements_reserve_both_paths_without_deadlock() {
    let parent = tempfile::tempdir().unwrap();
    let media = qualified(&parent.path().join("store"), MediaFaultSchedule::default());
    let records = ArtifactTreeDirectory::families().child("records").unwrap();
    let tree = media.artifact_tree();
    tree.create_directory(&records).unwrap();
    let left = records.file("left.data").unwrap();
    let right = records.file("right.data").unwrap();
    tree.write_new(&left, b"left").unwrap();
    tree.write_new(&right, b"right").unwrap();

    std::thread::scope(|scope| {
        let start = std::sync::Arc::new(std::sync::Barrier::new(3));
        let (sent, received) = std::sync::mpsc::sync_channel(2);
        let left_to_right = media.artifact_tree();
        let left_source = left.clone();
        let right_target = right.clone();
        let first_start = std::sync::Arc::clone(&start);
        let first_sent = sent.clone();
        let first = scope.spawn(move || {
            first_start.wait();
            let result = left_to_right.replace(&left_source, &right_target);
            first_sent.send(()).unwrap();
            result
        });
        let right_to_left = media.artifact_tree();
        let second_start = std::sync::Arc::clone(&start);
        let second = scope.spawn(move || {
            second_start.wait();
            let result = right_to_left.replace(&right, &left);
            sent.send(()).unwrap();
            result
        });
        start.wait();
        for _ in 0..2 {
            received
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("opposing replacement deadlocked");
        }
        first.join().unwrap().unwrap();
        second.join().unwrap().unwrap();
    });
    let left = std::fs::read(parent.path().join("store/families/records/left.data")).ok();
    let right = std::fs::read(parent.path().join("store/families/records/right.data")).ok();
    assert!(
        (left.as_deref() == Some(b"left".as_slice()) && right.is_none())
            || (left.is_none() && right.as_deref() == Some(b"right".as_slice())),
        "result must match one of the two legal serial histories"
    );
    media.close();
}
