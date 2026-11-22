//! Exact durable owner-record boundary kills for single-branch adoption
//! (armed after Home open) and the World-Released / Query-ReleaseBranch cut.
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

struct StageHit {
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

fn kill_at_stage_and_reopen(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    stage: &str,
) -> StageHit {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("home");
    predecessor(&loc, false);
    let (end, detail) = run_child(&loc, &format!("stage:{stage}"));
    assert_eq!(
        end,
        crate::killed_child::ChildEnd::KilledAtPark,
        "stage:{stage} must park and be killed"
    );
    assert_eq!(
        detail.trim(),
        format!("stage={stage}"),
        "child must announce the exact durable owner-record stage"
    );
    let home = open_home(&loc, false, execution, None);
    let summary = home
        .runtime()
        .runtime()
        .cold_adoption_readmission_summary_for_test();
    let recovery_rows = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .len();
    let seam = classify_single_branch(summary, recovery_rows);
    StageHit {
        seam,
        detail,
        _dir: dir,
        loc,
        home,
    }
}

fn resume_native_recovery_adoption(
    home: &Home,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &str,
) -> CompositeCommitIdentity {
    let host = home.runtime();
    let p0 = *host.owned_revision();
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
        .product_publication_recovery_page(None, NonZeroUsize::MIN)
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
    assert_eq!((adoption.source(), adoption.target()), (&p0, &p1));
    let commit = adoption.product_commit().clone();
    cleanup.unwrap_or_else(|cause| panic!("{label}: cleanup recovered adoption: {cause:?}"));
    drop(adoption);
    assert!(
        host.runtime()
            .product_publication_recovery_page(None, NonZeroUsize::MIN)
            .unwrap()
            .rows()
            .is_empty(),
        "{label}: recovery catalogue cleared after resume"
    );
    commit
}

struct SingleBranchControl {
    installed_commits: usize,
    branch_history_len: usize,
}

fn measure_uninterrupted_adoption(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> SingleBranchControl {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("control");
    predecessor(&loc, false);
    let home = open_home(&loc, false, execution, None);
    let performed = adopt_branch_to_p1(&home);
    let commit = performed.product_commit().clone();
    drop(performed);
    let installed_commits = home
        .runtime()
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits();
    let branch = home.runtime().current_world();
    let (branch_history_len, _) = assert_single_p0_to_p1_occurrence(
        &home,
        branch,
        None,
        ExpectedAdoptionCommit::Live(&commit),
        "control",
    );
    verify_p1_and_close(
        home,
        execution,
        &loc,
        installed_commits,
        branch_history_len,
        ExpectedAdoptionCommit::Live(&commit),
        "control",
    );
    SingleBranchControl {
        installed_commits,
        branch_history_len,
    }
}

fn verify_durable_record_boundary(
    before_stage: &str,
    expected_before: SingleBranchSeam,
    after_stage: &str,
    expected_after: SingleBranchSeam,
) {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let control = measure_uninterrupted_adoption(execution);

    // Prove the immediately preceding boundary has not crossed this record's durability fence.
    let before = kill_at_stage_and_reopen(execution, before_stage);
    assert_eq!(
        before.seam, expected_before,
        "before-side stage:{before_stage} must remain {expected_before:?} ({})",
        before.detail
    );
    before.home.close(&execution).unwrap().reclaim();

    // Prove immediately after this durable owner record commits, cold readmission sees `expected_after`.
    let hit = kill_at_stage_and_reopen(execution, after_stage);
    assert_eq!(
        hit.seam, expected_after,
        "after-side stage:{after_stage} must land in {expected_after:?} ({})",
        hit.detail
    );
    let label = format!("{expected_after:?} (stage:{after_stage})");
    eprintln!("C3.4 durable boundary {expected_before:?} -> {expected_after:?}: {}", hit.detail);
    let commit = match expected_after {
        SingleBranchSeam::BeforeAdoption => unreachable!(),
        SingleBranchSeam::PreOpenRetained | SingleBranchSeam::EmptyOpenReleased => {
            Some(adopt_branch_to_p1(&hit.home).product_commit().clone())
        }
        SingleBranchSeam::NativeRecoveryRetained => {
            Some(resume_native_recovery_adoption(&hit.home, &execution, &label))
        }
        SingleBranchSeam::PublishedDischarged | SingleBranchSeam::AlreadyCompleted => None,
    };
    let expected_commit = commit
        .as_ref()
        .map_or(ExpectedAdoptionCommit::Any, ExpectedAdoptionCommit::Live);
    verify_p1_and_close(
        hit.home,
        execution,
        &hit.loc,
        control.installed_commits,
        control.branch_history_len,
        expected_commit,
        &label,
    );
}

#[test]
fn first_barrier_pre_open_retained_resumes_once() {
    verify_durable_record_boundary(
        "before-retain",
        SingleBranchSeam::BeforeAdoption,
        "after-retain",
        SingleBranchSeam::PreOpenRetained,
    );
}
#[test]
fn first_barrier_empty_open_released_resumes_once() {
    verify_durable_record_boundary(
        "after-retain",
        SingleBranchSeam::PreOpenRetained,
        "after-world-open",
        SingleBranchSeam::EmptyOpenReleased,
    );
}
#[test]
fn first_barrier_native_recovery_retained_resumes_once() {
    verify_durable_record_boundary(
        "after-world-open",
        SingleBranchSeam::EmptyOpenReleased,
        "before-product-compare",
        SingleBranchSeam::NativeRecoveryRetained,
    );
}
#[test]
fn first_barrier_published_discharged_resumes_once() {
    verify_durable_record_boundary(
        "before-product-compare",
        SingleBranchSeam::NativeRecoveryRetained,
        "after-world-published",
        SingleBranchSeam::PublishedDischarged,
    );
}
#[test]
fn first_barrier_already_completed_resumes_once() {
    verify_durable_record_boundary(
        "after-world-published",
        SingleBranchSeam::PublishedDischarged,
        "after-obligation-released",
        SingleBranchSeam::AlreadyCompleted,
    );
}

#[test]
fn empty_open_world_released_cut_never_reports_adoption_success() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let count_dir = tempfile::tempdir().unwrap();
    let count_loc = count_dir.path().join("count");
    predecessor(&count_loc, false);
    assert_eq!(
        run_child(&count_loc, "stage:after-world-open").0,
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let (end, reopen_count_text) = run_child(&count_loc, "reopen-count");
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    let reopen_syncs: u64 = reopen_count_text.trim().parse().unwrap();

    let (mut lo, mut hi, mut hit_released_discharged) = (1u64, reopen_syncs + 2, false);
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let d = tempfile::tempdir().unwrap();
        let l = d.path().join("cut");
        predecessor(&l, false);
        assert_eq!(
            run_child(&l, "stage:after-world-open").0,
            crate::killed_child::ChildEnd::KilledAtPark
        );
        let (end, detail) = run_child(&l, &format!("reopen-kill:{mid}"));
        if end == crate::killed_child::ChildEnd::Finished {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
            continue;
        }
        assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark, "reopen mid={mid}");
        assert!(
            detail.contains("role=SynchronizeFileState")
                && detail.contains("path=Some(ArtifactOwned)")
                && detail.contains("op=Some(")
                && detail.contains("id_ord=Some("),
            "reopen barrier must report ArtifactOwned file sync context: {detail}"
        );
        let home = open_home(&l, false, execution, None);
        let s = home
            .runtime()
            .runtime()
            .cold_adoption_readmission_summary_for_test();
        assert_eq!(
            (s.published_discharged, s.already_completed),
            (0, 0),
            "no-effect release cannot report success (mid={mid}): {s:?}"
        );
        if s.released_discharged == 1 {
            eprintln!("C3.4 World-Released / Query-ReleaseBranch cut: {detail}");
            hit_released_discharged = true;
            home.close(&execution).unwrap().reclaim();
            let after = open_home(&l, false, execution, None);
            let s2 = after
                .runtime()
                .runtime()
                .cold_adoption_readmission_summary_for_test();
            assert_eq!(
                (s2.already_released, s2.already_completed, s2.published_discharged),
                (1, 0, 0),
                "discharged no-effect attempt must classify as already_released"
            );
            after.close(&execution).unwrap().reclaim();
            break;
        }
        let progressed_past = s.already_released == 1;
        home.close(&execution).unwrap().reclaim();
        if progressed_past {
            if mid == 0 {
                break;
            }
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
    assert!(
        hit_released_discharged,
        "must observe World Released / Query ReleaseBranch cut within {reopen_syncs} syncs"
    );
}
