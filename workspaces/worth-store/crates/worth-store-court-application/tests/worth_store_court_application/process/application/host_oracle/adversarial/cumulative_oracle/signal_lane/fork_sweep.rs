//! Tier-3 exhaustive `SynchronizeFileState` barrier sweep on a **forked**
//! Signal+Bridge authored product branch across alternating builds and D14
//! derived-state discards.
use super::child::CumSignalSession;
use super::ops::{
    assert_multi_parity, assert_receipt_bound, fork_on_home, kill_cum_signal_barrier,
    measure_cum_signal_barriers, observe_multi_branch, parse_barrier_finish, sorted_branches,
    CumSignalCmd,
};
use super::super::super::super::opening;
use super::super::super::dimensions_matrix::assert_cadence_discarded;
use super::super::super::signal_bridge_io_kills::{
    commit_mixed, open_signal_home, publish_definition, verify_recovered_bridge_definition,
    SignalOpen,
};
use super::super::super::{
    assert_barrier_killed, run_parallel_calibrated_barriers, BarrierKill, RestorableHome,
    BUILD_PRIMARY,
};
use super::super::{StepCtx, BUILDS};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::WorthQueryApplicationHomeLocation as Location;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::ContactCounters;

const FORK_BRANCH_IDX: usize = 1;
const TARGET_ORD: u8 = 77;
/// Multi-journal WAL group-commit coalescing bound on a 2-branch forked Signal Home
/// (especially after D14 derived-state discard where derived state rebuilds).
const FORK_TAIL_VARIANCE_BOUND: u64 = 12;

fn seed_forked_signal_home(loc: &std::path::Path, exec: &ExecutionRequest<'_, '_>) {
    let home = open_signal_home(
        SignalOpen {
            location: Location::Directory(loc.to_path_buf()),
            build: BUILD_PRIMARY,
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: ContactCounters::default(),
            placement: None,
        },
        exec,
    );
    fork_on_home(&home);
    let branches = sorted_branches(&home);
    assert!(commit_mixed(&home, branches[0], 11)
        .require_committed()
        .is_ok());
    publish_definition(&home, branches[FORK_BRANCH_IDX]);
    home.close(exec).unwrap().reclaim();
}

pub(in super::super) fn run_forked_signal_barrier_sweep() {
    let req = opening::serial();
    let exec = ExecutionRequest::serial(&req);
    let ref_home = open_signal_home(
        SignalOpen {
            location: Location::MemoryOnly,
            build: BUILD_PRIMARY,
            initialized: Arc::new(AtomicUsize::new(0)),
            contacts: ContactCounters::default(),
            placement: None,
        },
        &exec,
    );
    fork_on_home(&ref_home);
    let ref_branches = sorted_branches(&ref_home);
    assert!(commit_mixed(&ref_home, ref_branches[0], 11)
        .require_committed()
        .is_ok());
    publish_definition(&ref_home, ref_branches[FORK_BRANCH_IDX]);
    let expected_def_id =
        verify_recovered_bridge_definition(&ref_home, ref_branches[FORK_BRANCH_IDX]);
    let ref_before = observe_multi_branch(&ref_home);
    assert!(
        commit_mixed(&ref_home, ref_branches[FORK_BRANCH_IDX], TARGET_ORD)
            .require_committed()
            .is_ok()
    );
    let ref_after = observe_multi_branch(&ref_home);
    drop(ref_home.close(&exec).err().unwrap().resume().unwrap());

    let configs: [(&str, bool, &'static str, &'static str); 2] = [
        ("retained", false, BUILDS[1], BUILDS[2]),
        ("d14-discarded", true, BUILDS[2], BUILDS[0]),
    ];
    for (cfg_name, discard, kill_build, reopen_build) in configs {
        let saw_preop = Arc::new(AtomicBool::new(false));
        let saw_custody = Arc::new(AtomicBool::new(false));
        let saw_committed = Arc::new(AtomicBool::new(false));
        let label = format!("C4b.4 Tier-3 forked Signal barrier sweep ({cfg_name})");
        let _ = run_parallel_calibrated_barriers(
            &label,
            || {
                let req = opening::serial();
                let exec = ExecutionRequest::serial(&req);
                let home = RestorableHome::prepare(|loc| {
                    seed_forked_signal_home(loc, &exec);
                    if discard {
                        let ch = loc.parent().unwrap().join("cal-d14-ch");
                        let (mut s_d, _) = CumSignalSession::start(loc, &ch, BUILD_PRIMARY);
                        assert!(s_d
                            .send(CumSignalCmd::CheckpointAndDiscard)
                            .discard_gen
                            .is_some());
                        s_d.kill_live(&format!("fork-sweep-cal({cfg_name})"));
                    }
                });
                home.restore();
                let m1 = measure_cum_signal_barriers(
                    home.location(),
                    FORK_BRANCH_IDX,
                    TARGET_ORD,
                    kill_build,
                );
                home.restore();
                let m2 = measure_cum_signal_barriers(
                    home.location(),
                    FORK_BRANCH_IDX,
                    TARGET_ORD,
                    kill_build,
                );
                assert!(
                    m1.abs_diff(m2) <= FORK_TAIL_VARIANCE_BOUND,
                    "forked signal ({cfg_name}) calibration spread {m1} vs {m2}"
                );
                let w_total = m1.max(m2);
                assert!(
                    w_total >= 10,
                    "forked signal ({cfg_name}) must emit >= 10 syncs (got {w_total})"
                );
                (home, w_total)
            },
            |worker, nth, w_total| {
                let ch_root = tempfile::tempdir().unwrap();
                let ctx = StepCtx {
                    seed: 0,
                    step: nth as usize,
                    family: "signal+bridge-fork-sweep",
                    op: format!("forked-commit_mixed(branch={FORK_BRANCH_IDX},ord={TARGET_ORD})"),
                    cut: format!("SynchronizeFileState#{nth}"),
                    build: reopen_build,
                    derived: cfg_name,
                };
                let kill_ctx = StepCtx {
                    build: kill_build,
                    ..ctx.clone()
                };
                let kill_kind = if nth <= w_total.saturating_sub(FORK_TAIL_VARIANCE_BOUND) {
                    worker.restore();
                    kill_cum_signal_barrier(
                        worker.location(),
                        FORK_BRANCH_IDX,
                        TARGET_ORD,
                        kill_build,
                        nth,
                        &kill_ctx.to_string(),
                    );
                    BarrierKill::Killed
                } else {
                    let mut last_kind = None;
                    for _ in 0..8 {
                        worker.restore();
                        let markers = tempfile::tempdir().unwrap();
                        let spec = std::path::PathBuf::from(format!(
                            "{FORK_BRANCH_IDX}|{TARGET_ORD}|{nth}"
                        ));
                        let ready = markers.path().join("ready");
                        let fin = markers.path().join("finished");
                        let end = crate::killed_child::run_and_kill_with_ctx(
                            super::child::BARRIER_CHILD,
                            &[
                                (super::child::ENV_HOME, worker.location()),
                                (super::child::ENV_CH, markers.path()),
                                (super::child::ENV_BUILD, std::path::Path::new(kill_build)),
                                (super::child::ENV_BSPEC, &spec),
                            ],
                            &ready,
                            &fin,
                            &kill_ctx.to_string(),
                        );
                        if end == crate::killed_child::ChildEnd::KilledAtPark {
                            assert_barrier_killed(
                                end,
                                &ready,
                                worth_store::physical_runtime::certification::MediaOperationRole::SynchronizeFileState,
                                nth,
                                &kill_ctx.to_string(),
                            );
                            last_kind = Some(BarrierKill::Killed);
                            break;
                        }
                        let note = std::fs::read_to_string(&fin).unwrap_or_default();
                        let (actual, outcome) = parse_barrier_finish(&note).unwrap_or_else(|| {
                            panic!(
                                "{kill_ctx}: tail barrier finished without missed:actual=<N>:outcome=<tag> (note={note:?})"
                            )
                        });
                        assert_eq!(
                            outcome, "committed",
                            "{kill_ctx}: tail barrier #{nth} finished uncommitted ({outcome}) without parking (note={note:?})"
                        );
                        assert!(
                            actual < nth
                                && actual >= w_total.saturating_sub(FORK_TAIL_VARIANCE_BOUND),
                            "{kill_ctx}: barrier #{nth} missed unexpectedly (actual={actual}, w_total={w_total}, bound={FORK_TAIL_VARIANCE_BOUND})"
                        );
                        last_kind = Some(BarrierKill::CompletedAtTail { actual });
                    }
                    last_kind.unwrap()
                };
                let (mut s, o) = CumSignalSession::start(
                    worker.location(),
                    &ch_root.path().join("reopen"),
                    reopen_build,
                );
                assert_eq!((o.initialized, o.contacts), (0, (0, 0, 0, 0)), "{ctx}");
                if discard {
                    assert_cadence_discarded(&o.cadence, &ctx.to_string());
                }
                let snap = o.snap.as_ref().unwrap();
                if !o.custody.is_empty() {
                    assert_eq!(
                        kill_kind,
                        BarrierKill::Killed,
                        "{ctx}: custody posture requires verified parked OS kill (got {kill_kind:?})"
                    );
                    saw_custody.store(true, Ordering::SeqCst);
                    assert_eq!(o.custody.len(), 1, "{ctx}: 1 custody row");
                    assert_multi_parity(snap, &ref_before, &ctx);
                    let unpub = s.send(CumSignalCmd::CommitMixed {
                        branch_idx: FORK_BRANCH_IDX,
                        ordinal: TARGET_ORD,
                    });
                    assert_eq!(
                        unpub.commit_tag.as_deref(),
                        Some("product_unpublished"),
                        "{ctx}"
                    );
                    assert_eq!(unpub.contacts, (0, 0, 0, 0), "{ctx}");
                    let cont = s.send(CumSignalCmd::ContinueCustody);
                    assert_eq!(cont.contacts, (0, 0, 0, 0), "{ctx}");
                    let csnap = cont.snap.as_ref().unwrap();
                    assert_multi_parity(csnap, &ref_after, &ctx);
                    let r = s.send(CumSignalCmd::CommitMixed {
                        branch_idx: FORK_BRANCH_IDX,
                        ordinal: TARGET_ORD,
                    });
                    assert_eq!(
                        r.commit_tag.as_deref(),
                        Some("recovered_committed"),
                        "{ctx}"
                    );
                    assert_eq!(r.contacts, (0, 0, 0, 0), "{ctx}");
                    assert_receipt_bound(
                        r.receipt.as_ref().unwrap(),
                        TARGET_ORD,
                        &csnap.branches[FORK_BRANCH_IDX],
                        &ctx,
                    );
                } else if snap.branches[FORK_BRANCH_IDX].intent
                    == ref_before.branches[FORK_BRANCH_IDX].intent
                {
                    assert_eq!(
                        kill_kind,
                        BarrierKill::Killed,
                        "{ctx}: pre-op posture requires verified parked OS kill (got {kill_kind:?})"
                    );
                    saw_preop.store(true, Ordering::SeqCst);
                    assert_multi_parity(snap, &ref_before, &ctx);
                    let r = s.send(CumSignalCmd::CommitMixed {
                        branch_idx: FORK_BRANCH_IDX,
                        ordinal: TARGET_ORD,
                    });
                    assert_eq!(r.commit_tag.as_deref(), Some("committed"), "{ctx}");
                    let rsnap = r.snap.as_ref().unwrap();
                    assert_receipt_bound(
                        r.receipt.as_ref().unwrap(),
                        TARGET_ORD,
                        &rsnap.branches[FORK_BRANCH_IDX],
                        &ctx,
                    );
                    assert_multi_parity(rsnap, &ref_after, &ctx);
                } else {
                    if kill_kind == BarrierKill::Killed {
                        saw_committed.store(true, Ordering::SeqCst);
                    }
                    assert_multi_parity(snap, &ref_after, &ctx);
                    let r = s.send(CumSignalCmd::CommitMixed {
                        branch_idx: FORK_BRANCH_IDX,
                        ordinal: TARGET_ORD,
                    });
                    assert_eq!(
                        r.commit_tag.as_deref(),
                        Some("recovered_committed"),
                        "{ctx}"
                    );
                    assert_eq!(r.contacts, (0, 0, 0, 0), "{ctx}");
                    assert_receipt_bound(
                        r.receipt.as_ref().unwrap(),
                        TARGET_ORD,
                        &snap.branches[FORK_BRANCH_IDX],
                        &ctx,
                    );
                }
                let vb = s.send(CumSignalCmd::VerifyRecoveredBridge {
                    branch_idx: FORK_BRANCH_IDX,
                });
                assert_eq!(vb.definition_identity, Some(expected_def_id), "{ctx}");
                let closed = s.send(CumSignalCmd::Close {
                    expect_discarded: false,
                });
                assert!(closed.closed && closed.close_gen.is_some(), "{ctx}");
                s.finish();
            },
        );
        assert!(
            saw_preop.load(Ordering::SeqCst)
                && saw_custody.load(Ordering::SeqCst)
                && saw_committed.load(Ordering::SeqCst),
            "forked Signal sweep ({cfg_name}) must hit all 3 postures (preop={}, custody={}, committed={})",
            saw_preop.load(Ordering::SeqCst),
            saw_custody.load(Ordering::SeqCst),
            saw_committed.load(Ordering::SeqCst),
        );
    }
}
