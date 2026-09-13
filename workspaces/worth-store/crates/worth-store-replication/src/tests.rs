use crate::{DurabilityReplayIdentity, DurabilityReplayKind};
use worth_store_physical_backend::BackendTargetProfile;
use worth_store_security::{
    readmitted_foreign_wal_security_scope_for_test, readmitted_wal_security_scope_for_test,
};

mod storage_recovery;

use crate::{
    admit_replication_publication_readiness, admit_replication_source, ObserveReplicationAdmission,
    ReplicationAdmissionRuntime, ReplicationAdmissionStage, ReplicationCapsuleId,
    ReplicationDeliveryKind, ReplicationPeerCapacity, ReplicationProgressDenial,
    ReplicationProgressOutcomeView, ReplicationPublicationDenial,
    ReplicationPublicationOutcomeView, ReplicationSourceAdmissionDenial,
    ReplicationSourceAdmissionOutcomeView, ReplicationSourceDeclaration,
};

#[test]
fn hostile_epoch_lineage_and_overlap_never_issue_readiness() {
    let first = source(1, 7, "lineage-a", 10, 20, "sha256:first");
    let authority = first.security_scope().current_authority().clone();
    let directory = tempfile::tempdir().unwrap();
    let mut runtime = ReplicationAdmissionRuntime::bind(directory.path(), &authority);
    published_progress(&mut runtime, first);

    assert!(matches!(
        runtime
            .observe_progress(source(2, 8, "lineage-a", 20, 30, "sha256:copied-epoch"))
            .view(),
        ReplicationProgressOutcomeView::Denied(ReplicationProgressDenial::SourceEpochMismatch)
    ));
    assert!(matches!(
        runtime
            .observe_progress(source(3, 7, "lineage-b", 20, 30, "sha256:other-lineage"))
            .view(),
        ReplicationProgressOutcomeView::Denied(ReplicationProgressDenial::LineageDivergence)
    ));
    assert!(matches!(
        runtime
            .observe_progress(source(4, 7, "lineage-a", 15, 25, "sha256:overlap"))
            .view(),
        ReplicationProgressOutcomeView::Denied(ReplicationProgressDenial::DivergentReplayOverlap)
    ));
    assert!(matches!(
        runtime
            .observe_progress(source(5, 7, "lineage-a", 21, 30, "sha256:gap"))
            .view(),
        ReplicationProgressOutcomeView::Denied(ReplicationProgressDenial::ReplayProgressGap)
    ));
}

#[test]
fn duplicate_is_stale_and_resume_preserves_source_lineage() {
    let first = source(1, 7, "lineage-a", 10, 20, "sha256:first");
    let authority = first.security_scope().current_authority().clone();
    let directory = tempfile::tempdir().unwrap();
    let mut runtime = ReplicationAdmissionRuntime::bind(directory.path(), &authority);
    published_progress(&mut runtime, first);
    let duplicate_outcome =
        runtime.observe_progress(source(1, 7, "lineage-a", 10, 20, "sha256:first"));
    let ReplicationProgressOutcomeView::Duplicate(duplicate) = duplicate_outcome.view() else {
        panic!("exact duplicate must be an idempotent stale delivery")
    };
    assert_eq!(
        duplicate.observe_replication_admission().stage(),
        ReplicationAdmissionStage::Duplicate
    );

    let resumed = runtime.observe_progress(source(2, 7, "lineage-a", 20, 30, "sha256:second"));
    let progress = resumed.into_observed_progress().unwrap();
    assert_eq!(progress.delivery_kind(), ReplicationDeliveryKind::Resumed);
    assert_eq!(
        progress.observe_replication_admission().stage(),
        ReplicationAdmissionStage::ProgressObserved
    );
    let readiness = admit_replication_publication_readiness(progress);
    let authority = readiness
        .source()
        .security_scope()
        .current_authority()
        .clone();
    let published = runtime
        .publish(readiness, &authority)
        .into_result()
        .unwrap();
    assert_eq!(published.peer_progress().source_epoch().get(), 7);
    assert_eq!(published.peer_progress().lineage().as_str(), "lineage-a");
    assert_eq!(published.peer_progress().replay_identity().last_lsn(), 30);
}

#[test]
fn replay_declaration_cannot_substitute_for_observed_replay_identity() {
    let scope = readmitted_wal_security_scope_for_test();
    let authority = scope.current_authority().clone();
    let durable = publication(10, 20, "sha256:actual");
    let declaration = ReplicationSourceDeclaration::new(
        ReplicationCapsuleId(9),
        "peer-a",
        7,
        "lineage-a",
        "sha256:copied",
        10,
        20,
    );

    assert!(matches!(
        admit_replication_source(declaration, scope, &authority, durable).view(),
        ReplicationSourceAdmissionOutcomeView::Denied(
            ReplicationSourceAdmissionDenial::ReplayIdentityMismatch
        )
    ));
}

#[test]
fn current_authority_cannot_change_between_admission_and_publication() {
    let source = source(6, 7, "lineage-a", 30, 40, "sha256:authority");
    let authority = source.security_scope().current_authority().clone();
    let directory = tempfile::tempdir().unwrap();
    let mut runtime = ReplicationAdmissionRuntime::bind(directory.path(), &authority);
    let progress = runtime
        .observe_progress(source)
        .into_observed_progress()
        .unwrap();
    let readiness = admit_replication_publication_readiness(progress);
    let foreign = readmitted_foreign_wal_security_scope_for_test();

    assert!(matches!(
        runtime
            .publish(readiness, foreign.current_authority())
            .view(),
        ReplicationPublicationOutcomeView::Denied(
            ReplicationPublicationDenial::CurrentAuthorityChanged
        )
    ));
}

#[test]
fn stale_pending_observation_cannot_regress_peer_progress() {
    let first = source(7, 7, "lineage-a", 10, 20, "sha256:first-pending");
    let authority = first.security_scope().current_authority().clone();
    let directory = tempfile::tempdir().unwrap();
    let mut runtime = ReplicationAdmissionRuntime::bind(directory.path(), &authority);
    let first_readiness = admit_replication_publication_readiness(
        runtime
            .observe_progress(first)
            .into_observed_progress()
            .unwrap(),
    );
    let stale_readiness = admit_replication_publication_readiness(
        runtime
            .observe_progress(source(8, 7, "lineage-a", 10, 30, "sha256:stale-pending"))
            .into_observed_progress()
            .unwrap(),
    );

    runtime
        .publish(first_readiness, &authority)
        .into_result()
        .unwrap();

    assert!(matches!(
        runtime.publish(stale_readiness, &authority).view(),
        ReplicationPublicationOutcomeView::Denied(
            ReplicationPublicationDenial::PeerProgressChanged
        )
    ));
}

#[test]
fn durable_progress_reopens_as_resume_authority() {
    let directory = progress_directory("reopen");
    let first = source(20, 7, "lineage-a", 10, 20, "sha256:restart-first");
    let authority = first.security_scope().current_authority().clone();
    let mut runtime = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(4).unwrap(),
    )
    .unwrap();
    published_progress(&mut runtime, first);
    drop(runtime);

    let reopened = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(4).unwrap(),
    )
    .unwrap();
    let progress = reopened
        .observe_progress(source(21, 7, "lineage-a", 20, 30, "sha256:restart-second"))
        .into_observed_progress()
        .unwrap();
    assert_eq!(progress.delivery_kind(), ReplicationDeliveryKind::Resumed);
}

#[test]
fn alternating_snapshots_bound_progress_storage() {
    let directory = progress_directory("bounded");
    let first = source(30, 7, "lineage-a", 0, 1, "sha256:bounded-0");
    let authority = first.security_scope().current_authority().clone();
    let mut runtime = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(1).unwrap(),
    )
    .unwrap();
    published_progress(&mut runtime, first);
    for step in 1..20 {
        published_progress(
            &mut runtime,
            source(
                30 + step,
                7,
                "lineage-a",
                step,
                step + 1,
                &format!("sha256:bounded-{step}"),
            ),
        );
    }
    let snapshots = std::fs::read_dir(directory.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".snapshot"))
        .count();
    assert_eq!(snapshots, 2);
}

#[test]
fn progress_store_is_bound_to_current_authority() {
    let directory = progress_directory("authority");
    let first = source(60, 7, "lineage-a", 10, 20, "sha256:authority-first");
    let authority = first.security_scope().current_authority().clone();
    let mut runtime = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(1).unwrap(),
    )
    .unwrap();
    published_progress(&mut runtime, first);
    drop(runtime);
    let foreign = readmitted_foreign_wal_security_scope_for_test();
    assert_eq!(
        ReplicationAdmissionRuntime::open(
            directory.path(),
            foreign.current_authority(),
            ReplicationPeerCapacity::new(1).unwrap(),
        )
        .unwrap_err(),
        ReplicationPublicationDenial::CurrentAuthorityChanged,
    );
}

#[test]
fn torn_inactive_snapshot_falls_back_to_last_complete_generation() {
    let directory = progress_directory("torn-fallback");
    let first = source(70, 7, "lineage-a", 0, 1, "sha256:torn-first");
    let authority = first.security_scope().current_authority().clone();
    let mut runtime = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(1).unwrap(),
    )
    .unwrap();
    published_progress(&mut runtime, first);
    published_progress(
        &mut runtime,
        source(71, 7, "lineage-a", 1, 2, "sha256:torn-second"),
    );
    drop(runtime);
    std::fs::OpenOptions::new()
        .write(true)
        .open(directory.path().join("replication-progress-0.snapshot"))
        .unwrap()
        .set_len(8)
        .unwrap();

    let reopened = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(1).unwrap(),
    )
    .unwrap();
    let resumed = reopened.observe_progress(source(72, 7, "lineage-a", 1, 2, "sha256:torn-replay"));
    assert!(matches!(
        resumed.view(),
        ReplicationProgressOutcomeView::Observed(progress)
            if progress.delivery_kind() == ReplicationDeliveryKind::Resumed
    ));
}

#[test]
fn checksummed_snapshot_corruption_is_not_silently_downgraded() {
    let directory = progress_directory("corruption");
    let first = source(80, 7, "lineage-a", 0, 1, "sha256:corrupt-first");
    let authority = first.security_scope().current_authority().clone();
    let mut runtime = ReplicationAdmissionRuntime::open(
        directory.path(),
        &authority,
        ReplicationPeerCapacity::new(1).unwrap(),
    )
    .unwrap();
    published_progress(&mut runtime, first);
    published_progress(
        &mut runtime,
        source(81, 7, "lineage-a", 1, 2, "sha256:corrupt-second"),
    );
    drop(runtime);
    let path = directory.path().join("replication-progress-0.snapshot");
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.last_mut().unwrap();
    *last ^= 0xff;
    std::fs::write(path, bytes).unwrap();

    assert_eq!(
        ReplicationAdmissionRuntime::open(
            directory.path(),
            &authority,
            ReplicationPeerCapacity::new(1).unwrap(),
        )
        .unwrap_err(),
        ReplicationPublicationDenial::ProgressStoreIo,
    );
}

fn source(
    capsule: u64,
    epoch: u64,
    lineage: &str,
    first_lsn: u64,
    last_lsn: u64,
    digest: &str,
) -> crate::AdmittedReplicationSource {
    let scope = readmitted_wal_security_scope_for_test();
    let authority = scope.current_authority().clone();
    let declaration = ReplicationSourceDeclaration::new(
        ReplicationCapsuleId(capsule),
        "peer-a",
        epoch,
        lineage,
        digest,
        first_lsn,
        last_lsn,
    );
    admit_replication_source(
        declaration,
        scope,
        &authority,
        publication(first_lsn, last_lsn, digest),
    )
    .into_result()
    .unwrap()
}

fn publication(first_lsn: u64, last_lsn: u64, digest: &str) -> DurabilityReplayIdentity {
    DurabilityReplayIdentity::new(
        DurabilityReplayKind::WalFrame,
        BackendTargetProfile::PosixFileFsyncDirSync,
        digest,
        first_lsn,
        last_lsn,
    )
    .unwrap()
}

fn published_progress(
    runtime: &mut ReplicationAdmissionRuntime,
    source: crate::AdmittedReplicationSource,
) {
    let progress = runtime
        .observe_progress(source)
        .into_observed_progress()
        .unwrap();
    let readiness = admit_replication_publication_readiness(progress);
    let authority = readiness
        .source()
        .security_scope()
        .current_authority()
        .clone();
    runtime
        .publish(readiness, &authority)
        .into_result()
        .unwrap();
}

fn progress_directory(label: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("worth-store-replication-{label}-"))
        .tempdir()
        .unwrap()
}
