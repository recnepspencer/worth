#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::*;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRetirementDenial, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation, crate::durability_admission::managed_mutation::extent_record_rewrite::independent_arena_observer, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
use {std::path::Path, std::path::PathBuf, std::process::Command, std::process::Stdio, std::thread, std::time::Duration, std::time::Instant};
use {worth_store::physical_runtime::ArenaEvacuationThreshold, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordPlacementPolicy};
use {worth_store_physical_format::RecordArtifactFile};

#[test]
fn process_kills_converge_whole_arena_retirement() {
    use {worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
    for seam in [1_u8, 2, 3, 4] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("store");
        kill_at_seam(directory.path(), seam);
        assert_offline_arena_integrity(&root, directory.path(), seam, "killed");
        let arena = arena_file(&root);
        assert_eq!(arena.exists(), seam != 2, "seam {seam} file state");
        let records = produced_retirement_payloads(&root);
        let arena_intents = records
            .iter()
            .filter(|record| {
                record.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena
                    && record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent
            })
            .count();
        assert_eq!(
            arena_intents, 1,
            "seam {seam} keeps one durable original intent"
        );
        let serving = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement::reopen_with_copy_policy(&root);
        serving
            .retire_displaced_segment()
            .unwrap_or_else(|denial| panic!("seam {seam} retirement resume failed: {denial:?}"));
        assert!(!arena.exists(), "seam {seam} resumes exact arena deletion");
        assert!(serving.records().is_ok());
        serving.close();
        let records = produced_retirement_payloads(&root);
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena
                    && record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent)
                .count(),
            1
        );
        assert_eq!(
            records
                .iter()
                .filter(|record| record.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Arena
                    && record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion)
                .count(),
            1
        );
        let reopened = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement::reopen_with_copy_policy(&root);
        assert_eq!(
            reopened.retire_displaced_segment(),
            Err(worth_store::physical_runtime::PhysicalRetirementDenial::Absent)
        );
        reopened.close();
        assert_offline_arena_integrity(&root, directory.path(), seam, "recovered");
    }
}

#[test]
#[ignore = "spawned and killed at a real arena-retirement boundary"]
fn arena_retirement_child() {
    let directory = std::path::PathBuf::from(std::env::var_os(MARKER).unwrap());
    let seam: u8 = std::env::var(SEAM).unwrap().parse().unwrap();
    let root = directory.join("store");
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(worth_store::physical_runtime::ExtentArenaCapacity::bytes(64 << 20).unwrap())
        .arena_evacuation(worth_store::physical_runtime::ArenaEvacuationThreshold::percent(99).unwrap())
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = if std::env::var_os(SMALL_WAL).is_some() {
        worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(&media, small_wal_policy())
    } else {
        crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::fill_durability(&media)
    };
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    let payload = vec![61; 40_000];
    let append = completed(prepare(&serving, placement, [31; 32], &payload).execute());
    let identity = append.persisted_records()[0];
    let _record = append.into_acknowledgment().record_ids().next().unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([32; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), identity)
        .unwrap()
    else {
        panic!("selected copy was not prepared");
    };
    for _ in 0..1000 {
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption);
    completed(
        submission
            .prepare_completed_extent_copy()
            .unwrap()
            .execute(),
    );
    for ordinal in [1, 2] {
        let checkpoint = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::copy_checkpoint(&serving, ordinal);
        submission.finalize_extent_copy(&checkpoint).unwrap();
    }
    serving.retire_displaced_segment().unwrap();
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([33; 32]))
        .unwrap();
    assert!(matches!(
        submission
            .prepare_arena_evacuation(placement, request(key))
            .unwrap(),
        worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome::EmptyAwaitingRetirement { arena: 1, .. }
    ));
    assert_eq!(current_extent_route(&root, identity).arena, 2);
    let arrived = serving.certification_arm_retirement_kill(seam);
    std::thread::spawn(move || {
        while !arrived.load(std::sync::atomic::Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::fs::write(directory.join("reached"), b"1").unwrap();
    });
    let result = serving.retire_displaced_segment();
    panic!("armed arena retirement returned: {result:?}");
}
