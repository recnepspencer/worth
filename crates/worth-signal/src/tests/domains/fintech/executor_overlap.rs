use std::collections::BTreeMap;

use super::audit_surface::PrimaryAuditSurface;
use super::branch_checkpoint::BranchCheckpoint;
use super::fixture::FintechWorld;
use super::market_seed::MarketSeed;
use super::scenarios::setup_seeded_world;
use crate::facade::*;

fn capture(fixture: &mut FintechWorld, workers: Option<usize>) -> BranchCheckpoint {
    match workers {
        Some(workers) => fixture
            .capture_active_checkpoint_with_workers(workers)
            .unwrap(),
        None => fixture.capture_active_checkpoint().unwrap(),
    }
}

fn refresh(fixture: &mut FintechWorld, workers: Option<usize>) -> PrimaryAuditSurface {
    match workers {
        Some(workers) => fixture
            .refresh_primary_audit_surface_with_workers(workers)
            .unwrap(),
        None => fixture.refresh_primary_audit_surface().unwrap(),
    }
}

fn read(fixture: &mut FintechWorld, workers: Option<usize>) -> PrimaryAuditSurface {
    match workers {
        Some(workers) => fixture
            .read_primary_audit_surface_with_workers(workers)
            .unwrap(),
        None => fixture.read_primary_audit_surface().unwrap(),
    }
}

struct BranchDivergenceOutcome {
    main_audit: PrimaryAuditSurface,
    analysis_audit: PrimaryAuditSurface,
    correction_audit: PrimaryAuditSurface,
    analysis_replay: ReplaySlice,
    correction_replay: ReplaySlice,
    correction_lineage: Vec<LineageRecord>,
    branch_heads: BTreeMap<&'static str, Option<SignalSnapshotId>>,
}

fn run_drift_workflow(workers: Option<usize>) -> BranchDivergenceOutcome {
    let mut fixture = setup_seeded_world();
    let main_checkpoint = capture(&mut fixture, workers);

    let analysis = fixture.open_branch("analysis-drift").unwrap();
    fixture.seed_market(MarketSeed::high_vol(17)).unwrap();
    let analysis_audit = refresh(&mut fixture, workers);
    fixture.inject_primary_market_rollback().unwrap();
    let analysis_replay = fixture.replay_for_branch(analysis.clone());
    capture(&mut fixture, workers);

    fixture
        .switch_branch(main_checkpoint.branch.clone())
        .unwrap();
    fixture.restore_checkpoint(&main_checkpoint).unwrap();
    let correction = fixture.open_branch("correction-drift").unwrap();
    fixture.seed_market(MarketSeed::fx_dislocation(29)).unwrap();
    let correction_audit = refresh(&mut fixture, workers);
    fixture.inject_primary_market_rollback().unwrap();
    let correction_replay = fixture.replay_for_branch(correction.clone());
    capture(&mut fixture, workers);
    let correction_lineage = fixture.main_risk_lineage();

    fixture
        .switch_branch(main_checkpoint.branch.clone())
        .unwrap();
    fixture.restore_checkpoint(&main_checkpoint).unwrap();
    let main_audit = read(&mut fixture, workers);

    BranchDivergenceOutcome {
        main_audit,
        analysis_audit,
        correction_audit,
        analysis_replay,
        correction_replay,
        correction_lineage,
        branch_heads: BTreeMap::from([
            (
                "main",
                fixture.branch_head_snapshot_id(main_checkpoint.branch),
            ),
            ("analysis", fixture.branch_head_snapshot_id(analysis)),
            ("correction", fixture.branch_head_snapshot_id(correction)),
        ]),
    }
}

#[test]
fn fintech_serial_parallel_branch_divergence_keeps_overlap_honest_after_hostility() {
    let serial = run_drift_workflow(None);
    let checked_serial = run_drift_workflow(Some(1));
    assert_eq!(serial.main_audit, checked_serial.main_audit);
    assert_eq!(serial.analysis_audit, checked_serial.analysis_audit);
    assert_eq!(serial.correction_audit, checked_serial.correction_audit);
    assert_eq!(serial.branch_heads, checked_serial.branch_heads);

    // Ordinary reads settle dependencies one target at a time; checked reads
    // issue leased evaluations. Compare their audited values, then compare
    // exact replay identities among checked requests with the same boundaries.
    for workers in [2, 4] {
        let parallel = run_drift_workflow(Some(workers));

        assert_eq!(serial.main_audit, parallel.main_audit);
        assert_eq!(serial.analysis_audit, parallel.analysis_audit);
        assert_eq!(serial.correction_audit, parallel.correction_audit);
        assert_eq!(serial.branch_heads, parallel.branch_heads);

        let analysis_diff =
            compare_replay_slices(&checked_serial.analysis_replay, &parallel.analysis_replay);
        assert!(
            analysis_diff.mismatches.is_empty(),
            "workers={workers} analysis replay start={:?}/{:?} end={:?}/{:?} count={}/{}; first differing frames: {:?}",
            checked_serial.analysis_replay.start,
            parallel.analysis_replay.start,
            checked_serial.analysis_replay.end,
            parallel.analysis_replay.end,
            checked_serial.analysis_replay.frames.len(),
            parallel.analysis_replay.frames.len(),
            checked_serial
                .analysis_replay
                .frames
                .iter()
                .zip(&parallel.analysis_replay.frames)
                .enumerate()
                .find(|(_, (left, right))| left != right),
        );
        let correction_diff = compare_replay_slices(
            &checked_serial.correction_replay,
            &parallel.correction_replay,
        );
        assert!(
            correction_diff.mismatches.is_empty(),
            "workers={workers} correction replay start={:?}/{:?} end={:?}/{:?} count={}/{}; first differing frames: {:?}",
            checked_serial.correction_replay.start,
            parallel.correction_replay.start,
            checked_serial.correction_replay.end,
            parallel.correction_replay.end,
            checked_serial.correction_replay.frames.len(),
            parallel.correction_replay.frames.len(),
            checked_serial
                .correction_replay
                .frames
                .iter()
                .zip(&parallel.correction_replay.frames)
                .enumerate()
                .find(|(_, (left, right))| left != right),
        );
        let lineage_diff = compare_lineage_records(
            &checked_serial.correction_lineage,
            &parallel.correction_lineage,
        );
        assert!(
            lineage_diff.mismatches.is_empty(),
            "workers={workers} correction lineage count={}/{}; first differing records: {:?}",
            checked_serial.correction_lineage.len(),
            parallel.correction_lineage.len(),
            checked_serial
                .correction_lineage
                .iter()
                .zip(&parallel.correction_lineage)
                .enumerate()
                .find(|(_, (left, right))| left != right),
        );
    }
}
