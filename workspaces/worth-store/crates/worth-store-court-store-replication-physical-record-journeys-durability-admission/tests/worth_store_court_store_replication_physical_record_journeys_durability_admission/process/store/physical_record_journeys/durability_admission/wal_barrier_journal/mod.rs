#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::wal_barrier_journal::*;

use crate::physical_record_journeys::durability_admission::wal_barrier::append;
use worth_store_testkit_store::physical_record_journeys::configuration;

#[test]
fn wal_append_retires_its_record_durably_and_its_barrier_journals_nothing() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(
        &parent.path().join("store"),
    );
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    // The first commit also creates the WAL segment; measure a steady one.
    assert!(matches!(
        commit(&serving, 93),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let started = serving.media_counters();
    let appended = append(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([94; 32]),
    );

    let before = serving.media_counters();
    assert_eq!(
        (
            attempts(
                before,
                worth_store_physical_backend::MediaOperationRole::CreateNew
            ) - attempts(
                started,
                worth_store_physical_backend::MediaOperationRole::CreateNew
            ),
            attempts(
                before,
                worth_store_physical_backend::MediaOperationRole::Delete
            ) - attempts(
                started,
                worth_store_physical_backend::MediaOperationRole::Delete
            ),
            before.directory_syncs() - started.directory_syncs(),
        ),
        (1, 1, 2),
        "the append journals one record and retires it durably"
    );
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let acknowledged = serving.media_counters();
    let delta = |role| attempts(acknowledged, role) - attempts(before, role);
    assert_eq!(
        delta(worth_store_physical_backend::MediaOperationRole::CreateNew),
        0,
        "a WAL barrier writes no record"
    );
    assert_eq!(
        delta(worth_store_physical_backend::MediaOperationRole::Delete),
        0,
        "a WAL barrier retires no record"
    );
    assert_eq!(
        acknowledged.directory_syncs() - before.directory_syncs(),
        0,
        "a WAL barrier synchronizes no journal directory"
    );
    serving.close();
}

#[test]
fn failed_wal_flush_writes_its_record_and_reopen_fences_it() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&store_root)
        .close();
    let (serving, activations) = serving_with_faults(
        &store_root,
        &[(
            worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
            worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
        )],
    );
    assert_eq!(
        indeterminate_recovery(barrier_after_arming(&serving, &activations, 95)),
        worth_store::physical_runtime::PhysicalEffectRecoveryObligation::Retained
    );
    assert!(serving.close_plan().execute().requires_inspection());

    let reopened =
        worth_store_testkit_store::physical_record_journeys::serving_from_open(&store_root);
    let obligations = reopened.physical_recovery_obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(
        obligations[0].family(),
        worth_store::physical_runtime::PhysicalWorkOperationFamily::DurabilityBarrier
    );
    assert!(matches!(
        obligations[0].target(),
        worth_store::physical_runtime::PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
    ));
    assert_eq!(
        obligations[0].recovery_disposition(),
        worth_store::physical_runtime::PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert!(reopened.close_plan().execute().requires_inspection());
}

#[test]
fn failed_wal_flush_whose_record_write_also_fails_reports_both_failures() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&store_root)
        .close();
    let (serving, activations) = serving_with_faults(
        &store_root,
        &[
            (
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
                worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
            ),
            (
                worth_store_physical_backend::MediaOperationRole::CreateNew,
                worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
            ),
        ],
    );
    // The barrier's typed failure carries both: its outcome is indeterminate
    // and its recovery record is missing.
    assert_eq!(
        indeterminate_recovery(barrier_after_arming(&serving, &activations, 96)),
        worth_store::physical_runtime::PhysicalEffectRecoveryObligation::RetainedWithoutRecord
    );
    assert!(activations
        .iter()
        .all(worth_store_physical_backend::CertificationMediaFaultActivation::is_consumed));

    assert!(serving.close_plan().execute().requires_inspection());
    // Only the failed runtime fenced the flush: no record reaches reopen.
    let reopened =
        worth_store_testkit_store::physical_record_journeys::serving_from_open(&store_root);
    assert!(reopened.physical_recovery_obligations().is_empty());
    reopened.close();
}
