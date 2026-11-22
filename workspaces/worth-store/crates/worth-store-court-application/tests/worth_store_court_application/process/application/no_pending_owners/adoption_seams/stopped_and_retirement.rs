//! Two-branch stopped-prefix kills with preparation-entry observation and
//! per-branch durable activation occurrence parity.
use super::*;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionProgress as Progress, WorthQueryBranchSetAdoptionResumeDenial,
};
use worth_query_testkit_document_retention::facade::document_retention_model::host::SEED_RETENTION;

fn two_branches(home: &Home) -> (WorthQueryProductBranch, WorthQueryProductBranch) {
    let host = home.runtime();
    let root = host.current_world();
    let inventory = host
        .with_host_advancement(|phase| host.runtime().branches().inventory(&phase).unwrap())
        .unwrap();
    assert_eq!(inventory.iter().len(), 2, "predecessor created two branches");
    let child = inventory.iter().find(|b| *b != root).unwrap();
    (root, child)
}

pub(super) fn child_stopped_prefix(
    location: &Path,
    markers: &Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    advance_first_branch: bool,
) -> ! {
    let home = open_home(location, false, execution, None);
    let (a, b) = two_branches(&home);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let target = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let mut set = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    let stale_branch = if advance_first_branch { b } else { a };
    assert_eq!(settle(set_retention(host, stale_branch, 8, 0xB501)), RetentionVerdict::Performed(8));
    if advance_first_branch {
        let Some(Progress::Performed { branch, .. }) =
            set.advance(host.runtime(), &principal, &scope).unwrap()
        else {
            panic!("first branch must publish before stopping on branch 1");
        };
        assert_eq!(*branch, a);
    }
    let Some(Progress::ProductUnpublished { branch, adoption }) =
        set.advance(host.runtime(), &principal, &scope).unwrap()
    else {
        panic!("moved branch must stop the set");
    };
    assert_eq!(*branch, stale_branch);
    assert_eq!(
        adoption.before_effect_refusal(),
        Some(worth_query_host::facade::runtime::NoEffectCause::StaleExpectedProductHead)
    );
    let _stopped = set
        .begin_resume(host.runtime(), &scope)
        .unwrap_or_else(|(cause, _)| panic!("RetainStopped must publish: {cause:?}"));
    crate::killed_child::announce(&markers.join("ready"), b"stopped prefix durable");
    crate::killed_child::park();
}

struct ControlReference {
    installed_commits: usize,
    prep_work_units: usize,
    retentions: [u64; 2],
    branch_history_lens: [usize; 2],
}

fn uninterrupted_two_branch_control(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    edit_second_branch: bool,
) -> ControlReference {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("control");
    predecessor(&loc, true);
    let home = open_home(&loc, false, execution, None);
    let (a, b) = two_branches(&home);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let p0 = *host.owned_revision();
    let target = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
    let edited = if edit_second_branch { b } else { a };
    assert_eq!(settle(set_retention(host, edited, 8, 0xB501)), RetentionVerdict::Performed(8));
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let prep_observer = host.runtime().observe_branch_adoption_preparations_for_test();
    let mut set = host
        .runtime()
        .request(&principal, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    assert_eq!(prep_observer.recorded_branches(), vec![a, b], "control preparation enters both branches");
    drop(prep_observer);
    let prep_work_units = set.total_selection_work_units();
    assert!(prep_work_units > 0, "preparing two branches consumes selection work");
    let mut commits = Vec::new();
    for expected in [a, b] {
        let Some(Progress::Performed { branch, adoption }) =
            set.advance(host.runtime(), &principal, &scope).unwrap()
        else {
            panic!("control branch must publish");
        };
        assert_eq!(*branch, expected);
        assert_eq!((adoption.source(), adoption.target()), (&p0, &target));
        commits.push(adoption.product_commit().clone());
    }
    assert!(set.advance(host.runtime(), &principal, &scope).unwrap().is_none());
    drop(set.close().unwrap_or_else(|_| panic!("control set closes")));
    let (len_a, _) = assert_single_p0_to_p1_occurrence(&home, a, None, ExpectedAdoptionCommit::Live(&commits[0]), "control-a");
    let (len_b, _) = assert_single_p0_to_p1_occurrence(&home, b, None, ExpectedAdoptionCommit::Live(&commits[1]), "control-b");
    let installed_commits = host.runtime().world_history_snapshot_for_test().installed_commits();
    let retentions = [read_retention(host.runtime(), a), read_retention(host.runtime(), b)];
    home.close(&execution).unwrap().reclaim();
    ControlReference {
        installed_commits,
        prep_work_units,
        retentions,
        branch_history_lens: [len_a, len_b],
    }
}

#[test]
fn two_branch_set_stopped_prefix_killed_at_each_branch_resumes_suffix_once() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    for (mode, performed_before_stop) in [("stopped-branch-0", 0_usize), ("stopped-branch-1", 1)] {
        let control = uninterrupted_two_branch_control(execution, performed_before_stop == 1);
        let expected_retentions = if performed_before_stop == 0 { [8, SEED_RETENTION] } else { [SEED_RETENTION, 8] };
        assert_eq!(control.retentions, expected_retentions, "{mode}: control seed parity");

        let dir = tempfile::tempdir().unwrap();
        let loc = dir.path().join("home");
        predecessor(&loc, true);
        assert_eq!(run_child(&loc, mode).0, crate::killed_child::ChildEnd::KilledAtPark, "{mode}");

        let home = open_home(&loc, false, execution, None);
        let summary = home.runtime().runtime().cold_adoption_readmission_summary_for_test();
        assert_eq!((summary.stopped_sets_retained, summary.published_discharged), (1, 0), "{mode}");
        let (a, b) = two_branches(&home);
        let host = home.runtime();
        let p0 = *host.owned_revision();
        let p1 = *host.supported_program::<RetentionProgramP1>().unwrap().owned_revision();
        let p0_digest = CanonicalDigestId::new(*p0.as_bytes());
        let p1_digest = CanonicalDigestId::new(*p1.as_bytes());
        let expected_remaining = if performed_before_stop == 0 { vec![a, b] } else { vec![b] };

        let retire_refusal = host.retire_program_support(&execution, &p1);
        if performed_before_stop == 0 {
            assert!(matches!(
                retire_refusal,
                Err(WorthQueryProgramSupportRetirementFailure::Policy(
                    WorthQueryProgramSupportRetirementDenial::MandatoryCustody(ref inv)
                )) if inv.mandatory_custody() >= 1
            ), "{mode}: {retire_refusal:?}");
            assert_branch_revision(&home, a, &p0, mode);
            assert_branch_revision(&home, b, &p0, mode);
        } else {
            assert!(matches!(
                retire_refusal,
                Err(WorthQueryProgramSupportRetirementFailure::Policy(
                    WorthQueryProgramSupportRetirementDenial::CurrentBranches(ref inv)
                )) if inv.current_branches() == 1 && inv.mandatory_custody() >= 1
            ), "{mode}: {retire_refusal:?}");
            assert_branch_revision(&home, a, &p1, mode);
            assert_branch_revision(&home, b, &p0, mode);
        }

        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let mut stopped_list = host
            .with_host_advancement(|phase| host.runtime().readmit_stopped_branch_set_adoptions(&phase).unwrap())
            .unwrap();
        assert_eq!(stopped_list.len(), 1, "{mode}");
        let mut stopped = stopped_list.pop().unwrap();
        assert_eq!(stopped.target(), &p1, "{mode}: exact recorded target");
        assert_eq!(stopped.remaining_branches(), expected_remaining.as_slice(), "{mode}: ordered suffix");

        let prefix_commit = if performed_before_stop == 1 {
            let [Progress::RecordedPerformed { branch: br0, commit, source, target }, Progress::RecordedNoEffect { branch: br1, .. }] =
                stopped.progress()
            else {
                panic!("{mode}: expected recorded prefix + stop");
            };
            assert_eq!((*br0, *br1, *source, *target), (a, b, p0_digest, p1_digest), "{mode}");
            let prefix = *commit;
            let widened = host.runtime().branches().program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap()).unwrap();
            let failure = host
                .runtime()
                .request(&principal, &scope)
                .on_branches(widened, &[a, b])
                .unwrap()
                .programs()
                .resume(stopped, 64)
                .err()
                .expect("widened coverage must refuse re-preparing the stopped prefix");
            assert!(matches!(failure.denial(), WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch));
            stopped = failure.into_adoption();
            Some(prefix)
        } else {
            let [Progress::RecordedNoEffect { branch: br0, .. }] = stopped.progress() else {
                panic!("{mode}: expected single recorded stop on branch 0");
            };
            assert_eq!(*br0, a, "{mode}: stopped selector must be branch a");
            None
        };

        let work_before_resume = stopped.total_selection_work_units();
        let coverage = host
            .runtime()
            .branches()
            .program_adoption_coverage(&expected_remaining, NonZeroUsize::new(expected_remaining.len()).unwrap())
            .unwrap();
        // Directly observe branch selectors entering actual single-branch preparation.
        let prep_observer = host.runtime().observe_branch_adoption_preparations_for_test();
        let mut set = host
            .runtime()
            .request(&principal, &scope)
            .on_branches(coverage, &expected_remaining)
            .unwrap()
            .programs()
            .resume(stopped, 64)
            .unwrap_or_else(|failure| panic!("{mode}: resume suffix: {:?}", failure.denial()));
        assert_eq!(
            prep_observer.recorded_branches(),
            expected_remaining,
            "{mode}: completed prefix must never enter actual preparation"
        );
        drop(prep_observer);
        assert_eq!(set.pending_branches().collect::<Vec<_>>(), expected_remaining, "{mode}: exact pending suffix");
        let resumed_work = set.total_selection_work_units() - work_before_resume;
        assert_eq!(resumed_work * 2, control.prep_work_units * expected_remaining.len(), "{mode}: selection work matches suffix");

        let mut resumed_commits = Vec::new();
        for &expected in &expected_remaining {
            let Some(Progress::Performed { branch, adoption }) =
                set.advance(host.runtime(), &principal, &scope).unwrap()
            else {
                panic!("{mode}: resumed suffix branch must publish");
            };
            assert_eq!(*branch, expected);
            assert_eq!((adoption.source(), adoption.target()), (&p0, &p1), "{mode}: exact resumed transition");
            resumed_commits.push(adoption.product_commit().clone());
        }
        assert!(set.advance(host.runtime(), &principal, &scope).unwrap().is_none());
        let closed = set.close().unwrap_or_else(|_| panic!("{mode}: close set"));
        assert_eq!(closed.progress().len(), 2, "{mode}");
        assert_eq!([closed.progress()[0].branch(), closed.progress()[1].branch()], [a, b], "{mode}");
        if let Some(prefix) = prefix_commit {
            assert!(matches!(
                &closed.progress()[0],
                Progress::RecordedPerformed { branch, commit, source, target }
                    if *branch == a && *commit == prefix && *source == p0_digest && *target == p1_digest
            ));
            let performed_b = closed.progress()[1].performed().expect("suffix progress retains live receipt");
            assert_eq!((performed_b.source(), performed_b.target(), performed_b.product_commit()), (&p0, &p1, &resumed_commits[0]));
        } else {
            for (idx, commit) in resumed_commits.iter().enumerate() {
                let performed = closed.progress()[idx].performed().expect("resumed progress retains live receipt");
                assert_eq!((performed.source(), performed.target(), performed.product_commit()), (&p0, &p1, commit));
            }
        }
        drop(closed);

        // Before retiring P0 or writing probes: tie receipts to branch-local durable activation history.
        let expected_a = match prefix_commit {
            Some(prefix) => ExpectedAdoptionCommit::Durable(prefix),
            None => ExpectedAdoptionCommit::Live(&resumed_commits[0]),
        };
        let expected_b = ExpectedAdoptionCommit::Live(resumed_commits.last().unwrap());
        assert_single_p0_to_p1_occurrence(&home, a, Some(control.branch_history_lens[0]), expected_a, mode);
        assert_single_p0_to_p1_occurrence(&home, b, Some(control.branch_history_lens[1]), expected_b, mode);
        assert_eq!(host.runtime().world_history_snapshot_for_test().installed_commits(), control.installed_commits, "{mode}");
        assert_eq!([read_retention(host.runtime(), a), read_retention(host.runtime(), b)], control.retentions, "{mode}");
        assert_branch_revision(&home, a, &p1, mode);
        assert_branch_revision(&home, b, &p1, mode);
        assert!(host.retire_program_support(&execution, &p0).is_ok(), "{mode}: P0 retires");

        for (idx, branch) in [a, b].into_iter().enumerate() {
            assert_eq!(settle(set_retention(host, branch, 3, 0xB600 + idx as u64)), RetentionVerdict::violated("document-retention-v2"), "{mode}");
            let target_val = 15 + idx as u64;
            assert_eq!(settle(set_retention(host, branch, target_val, 0xB700 + idx as u64)), RetentionVerdict::Performed(target_val), "{mode}");
        }
        home.close(&execution).unwrap().reclaim();

        let reopened = open_home(&loc, false, execution, None);
        assert_eq!(
            [
                read_retention(reopened.runtime().runtime(), two_branches(&reopened).0),
                read_retention(reopened.runtime().runtime(), two_branches(&reopened).1),
            ],
            [15, 16],
            "{mode}: per-branch P1 values persist across reopen"
        );
        assert!(reopened
            .runtime()
            .with_host_advancement(|phase| reopened.runtime().runtime().readmit_stopped_branch_set_adoptions(&phase).unwrap())
            .unwrap()
            .is_empty());
        reopened.close(&execution).unwrap().reclaim();
    }
}
