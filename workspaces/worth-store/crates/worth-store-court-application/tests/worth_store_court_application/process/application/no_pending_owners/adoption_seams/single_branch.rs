//! First-durable-barrier single-branch adoption kills (armed after Home open)
//! and the World-Released / Query-ReleaseBranch observer cut.
use super::*;
use worth_query_execution::facade::application_home_integration::QueryColdAdoptionReadmissionSummary;
use worth_query_host::facade::application_entry::WorthQueryBranchAdoptionRecoveryOutcome;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SingleBranchSeam {
    BeforeAdoption = 0,
    PreOpenRetained = 1,
    EmptyOpenReleased = 2,
    NativeRecoveryRetained = 3,
    PublishedDischarged = 4,
    AlreadyCompleted = 5,
}

struct BarrierHit {
    nth: u64,
    seam: SingleBranchSeam,
    detail: String,
    _dir: tempfile::TempDir,
    loc: std::path::PathBuf,
    home: Home,
}

fn classify_single_branch(
    summary: QueryColdAdoptionReadmissionSummary,
    recovery_rows: usize,
) -> SingleBranchSeam {
    assert_eq!(
        (
            summary.released_discharged,
            summary.continued_discharged,
            summary.already_released,
            summary.stopped_sets_retained,
        ),
        (0, 0, 0, 0),
        "single-branch adoption cut cannot report unrelated release/stopped buckets: {summary:?}"
    );
    match (
        summary.pre_open_retained,
        summary.empty_open_released,
        summary.native_recovery_retained,
        summary.published_discharged,
        summary.already_completed,
        recovery_rows,
    ) {
        (0, 0, 0, 0, 0, 0) => SingleBranchSeam::BeforeAdoption,
        (1, 0, 0, 0, 0, 0) => SingleBranchSeam::PreOpenRetained,
        (0, 1, 0, 0, 0, 0) => SingleBranchSeam::EmptyOpenReleased,
        (0, 0, 1, 0, 0, 1) => SingleBranchSeam::NativeRecoveryRetained,
        (0, 0, 0, 1, 0, 0) => SingleBranchSeam::PublishedDischarged,
        (0, 0, 0, 0, 1, 0) => SingleBranchSeam::AlreadyCompleted,
        other => panic!("unexpected cold adoption summary/recovery tuple: {other:?} ({summary:?})"),
    }
}

fn close_hit(hit: BarrierHit, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    hit.home.close(execution).unwrap().reclaim();
}

fn probe_barrier(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    nth: u64,
) -> Option<BarrierHit> {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("probe");
    predecessor(&loc, false);
    let (end, detail) = run_child(&loc, &format!("single:{nth}"));
    if end == crate::killed_child::ChildEnd::Finished {
        return None;
    }
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "adoption barrier nth={nth} must park"
    );
    assert!(
        detail.contains("role=SynchronizeFileState")
            && detail.contains("op=Some(")
            && detail.contains("path=Some(")
            && detail.contains("id_ord=Some("),
        "nth={nth} must bind semantic media context: {detail}"
    );
    let home = open_home(&loc, false, execution, None);
    let summary = home
        .runtime()
        .runtime()
        .cold_adoption_readmission_summary_for_test();
    let recovery_rows = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .len();
    let seam = classify_single_branch(summary, recovery_rows);
    Some(BarrierHit {
        nth,
        seam,
        detail,
        _dir: dir,
        loc,
        home,
    })
}

fn find_first_barrier(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    mut lo: u64,
    mut hi: u64,
    target: SingleBranchSeam,
    expected_prev: SingleBranchSeam,
) -> BarrierHit {
    let mut best: Option<BarrierHit> = None;
    let mut prev_seen: Option<(u64, SingleBranchSeam)> = None;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let Some(hit) = probe_barrier(execution, mid) else {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
            continue;
        };
        if hit.seam >= target {
            if hit.seam == target && best.as_ref().is_none_or(|b| hit.nth < b.nth) {
                if let Some(old) = best.replace(hit) {
                    close_hit(old, &execution);
                }
            } else {
                close_hit(hit, &execution);
            }
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        } else {
            if prev_seen.is_none_or(|(n, _)| hit.nth > n) {
                prev_seen = Some((hit.nth, hit.seam));
            }
            close_hit(hit, &execution);
            lo = mid + 1;
        }
    }
    let mut first = best.unwrap_or_else(|| {
        panic!("must locate a real first barrier for {target:?} (lo={lo} hi={hi})")
    });
    // Confirm the immediately preceding barrier is strictly earlier; if ±1
    // background-sync jitter exposes an even earlier hit in `target`, step down.
    while first.nth > 1 {
        if let Some((prev_n, prev_seam)) = prev_seen
            && prev_n + 1 == first.nth
        {
            assert_eq!(
                prev_seam, expected_prev,
                "barrier {} before first {target:?} (nth={}) must be {expected_prev:?}",
                prev_n, first.nth
            );
            break;
        }
        let prev = probe_barrier(execution, first.nth - 1)
            .unwrap_or_else(|| panic!("barrier {} before first {target:?} must park", first.nth - 1));
        if prev.seam == target {
            let old = std::mem::replace(&mut first, prev);
            close_hit(old, &execution);
            continue;
        }
        assert_eq!(
            prev.seam, expected_prev,
            "barrier {} before first {target:?} (nth={}) must be {expected_prev:?}",
            prev.nth, first.nth
        );
        close_hit(prev, &execution);
        break;
    }
    eprintln!("C3.4 first barrier {target:?}: {}", first.detail);
    first
}

fn resume_native_recovery_adoption(
    home: &Home,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) {
    let host = home.runtime();
    let p1 = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    assert!(
        matches!(
            host.retire_program_support(execution, &p1),
            Err(WorthQueryProgramSupportRetirementFailure::Policy(
                WorthQueryProgramSupportRetirementDenial::MandatoryCustody(ref inv)
            )) if inv.mandatory_custody() >= 1
        ),
        "{label}: retained adoption recovery must protect target program support"
    );
    let page = host
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1, "{label}: one adoption recovery row");
    let recovery = host
        .runtime()
        .readmit_branch_adoption(page.rows()[0].handle())
        .unwrap_or_else(|cause| panic!("{label}: readmit branch adoption: {cause:?}"));
    assert_eq!(recovery.target(), &p1, "{label}: exact target revision");
    drop(page);
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("{label}: recover branch adoption"))
    else {
        panic!("{label}: recovered adoption must publish");
    };
    assert_eq!(adoption.target(), &p1);
    cleanup.unwrap_or_else(|cause| panic!("{label}: cleanup recovered adoption: {cause:?}"));
    drop(adoption);
    assert!(
        host.runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
            .unwrap()
            .rows()
            .is_empty(),
        "{label}: recovery catalogue cleared after resume"
    );
}

fn measure_uninterrupted_adoption(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> (u64, usize) {
    let control_dir = tempfile::tempdir().unwrap();
    let control_loc = control_dir.path().join("control");
    predecessor(&control_loc, false);
    let (end, count_text) = run_child(&control_loc, "single-uninterrupted");
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let total_syncs: u64 = count_text
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("expected identified sync count, got {count_text:?}"));
    assert!(total_syncs >= 5, "adoption performs at least 5 identified syncs: {total_syncs}");
    let control_home = open_home(&control_loc, false, execution, None);
    let control_commits = control_home
        .runtime()
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits();
    verify_p1_and_close(control_home, execution, &control_loc, control_commits, "control");
    (total_syncs, control_commits)
}

fn verify_first_durable_barrier(target: SingleBranchSeam, expected_prev: SingleBranchSeam) {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (total_syncs, control_commits) = measure_uninterrupted_adoption(execution);
    let hit = if target == SingleBranchSeam::PreOpenRetained {
        let first = probe_barrier(execution, 1).expect("nth=1 must park");
        if first.seam == target {
            eprintln!("C3.4 first barrier {target:?}: {}", first.detail);
            first
        } else {
            close_hit(first, &execution);
            find_first_barrier(execution, 2, total_syncs, target, expected_prev)
        }
    } else {
        find_first_barrier(execution, 1, total_syncs, target, expected_prev)
    };
    let label = format!("{target:?} (first barrier nth={})", hit.nth);
    match target {
        SingleBranchSeam::BeforeAdoption => unreachable!(),
        SingleBranchSeam::PreOpenRetained | SingleBranchSeam::EmptyOpenReleased => {
            adopt_branch_to_p1(&hit.home);
        }
        SingleBranchSeam::NativeRecoveryRetained => {
            resume_native_recovery_adoption(&hit.home, &execution, &label);
        }
        SingleBranchSeam::PublishedDischarged | SingleBranchSeam::AlreadyCompleted => {}
    }
    verify_p1_and_close(hit.home, execution, &hit.loc, control_commits, &label);
}

#[test]
fn first_barrier_pre_open_retained_resumes_once() {
    verify_first_durable_barrier(SingleBranchSeam::PreOpenRetained, SingleBranchSeam::BeforeAdoption);
}
#[test]
fn first_barrier_empty_open_released_resumes_once() {
    verify_first_durable_barrier(SingleBranchSeam::EmptyOpenReleased, SingleBranchSeam::PreOpenRetained);
}
#[test]
fn first_barrier_native_recovery_retained_resumes_once() {
    verify_first_durable_barrier(SingleBranchSeam::NativeRecoveryRetained, SingleBranchSeam::EmptyOpenReleased);
}
#[test]
fn first_barrier_published_discharged_resumes_once() {
    verify_first_durable_barrier(SingleBranchSeam::PublishedDischarged, SingleBranchSeam::NativeRecoveryRetained);
}
#[test]
fn first_barrier_already_completed_resumes_once() {
    verify_first_durable_barrier(SingleBranchSeam::AlreadyCompleted, SingleBranchSeam::PublishedDischarged);
}

#[test]
fn empty_open_world_released_cut_never_reports_adoption_success() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (total_syncs, _) = measure_uninterrupted_adoption(execution);
    let empty_hit = find_first_barrier(
        execution, 1, total_syncs,
        SingleBranchSeam::EmptyOpenReleased, SingleBranchSeam::PreOpenRetained,
    );
    let empty_setup = empty_hit.nth + 3;
    close_hit(empty_hit, &execution);

    let count_dir = tempfile::tempdir().unwrap();
    let count_loc = count_dir.path().join("count");
    predecessor(&count_loc, false);
    assert_eq!(run_child(&count_loc, &format!("single:{empty_setup}")).0, crate::killed_child::ChildEnd::KilledAtPark);
    let (end, reopen_count_text) = run_child(&count_loc, "reopen-count");
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let reopen_syncs: u64 = reopen_count_text.trim().parse().unwrap();

    let (mut lo, mut hi, mut hit_released_discharged) = (1u64, reopen_syncs + 2, false);
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let d = tempfile::tempdir().unwrap();
        let l = d.path().join("cut");
        predecessor(&l, false);
        assert_eq!(run_child(&l, &format!("single:{empty_setup}")).0, crate::killed_child::ChildEnd::KilledAtPark);
        let (end, detail) = run_child(&l, &format!("reopen-kill:{mid}"));
        if end == crate::killed_child::ChildEnd::Finished {
            if mid == 0 { break; }
            hi = mid - 1;
            continue;
        }
        assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark, "reopen mid={mid}");
        let home = open_home(&l, false, execution, None);
        let s = home.runtime().runtime().cold_adoption_readmission_summary_for_test();
        assert_eq!((s.published_discharged, s.already_completed), (0, 0), "no-effect release cannot report success (mid={mid}): {s:?}");
        if s.released_discharged == 1 {
            eprintln!("C3.4 World-Released / Query-ReleaseBranch cut: {detail}");
            hit_released_discharged = true;
            home.close(&execution).unwrap().reclaim();
            let after = open_home(&l, false, execution, None);
            let s2 = after.runtime().runtime().cold_adoption_readmission_summary_for_test();
            assert_eq!((s2.already_released, s2.already_completed, s2.published_discharged), (1, 0, 0), "discharged no-effect attempt must classify as already_released");
            after.close(&execution).unwrap().reclaim();
            break;
        }
        let progressed_past = s.already_released == 1;
        home.close(&execution).unwrap().reclaim();
        if progressed_past {
            if mid == 0 { break; }
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
    assert!(hit_released_discharged, "must observe World Released / Query ReleaseBranch cut within {reopen_syncs} syncs");
}
