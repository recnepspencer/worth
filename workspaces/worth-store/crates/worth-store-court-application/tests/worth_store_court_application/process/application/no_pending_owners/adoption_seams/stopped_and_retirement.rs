//! Two-branch stopped-prefix kills and Phase 6 support retirement across a real kill.
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryBranchSetAdoptionProgress as Progress,
        WorthQueryBranchSetAdoptionResumeDenial,
    },
    primary_graph::{WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted},
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::SEED_RETENTION,
    programs::RemovedOperationRetentionProgram,
    readback::observe_head,
};

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
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], std::num::NonZeroUsize::new(2).unwrap())
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
    assert_eq!(
        settle(set_retention(host, stale_branch, 8, 0xB501)),
        RetentionVerdict::Performed(8)
    );
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

pub(super) fn child_retire_and_kill(
    location: &Path,
    markers: &Path,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> ! {
    let home = open_home(location, false, execution, None);
    let host = home.runtime();
    let p0 = *host.owned_revision();
    let removal = *host
        .supported_program::<RemovedOperationRetentionProgram>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs();
    let requirements = programs.compare(&removal).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    host.retire_program_support(&execution, &p0)
        .expect("P0 retires after root adopts removal program");
    crate::killed_child::announce(&markers.join("ready"), b"retirement durable");
    crate::killed_child::park();
}

fn uninterrupted_two_branch_control(
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> usize {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("control");
    predecessor(&loc, true);
    let home = open_home(&loc, false, execution, None);
    let (a, b) = two_branches(&home);
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], std::num::NonZeroUsize::new(2).unwrap())
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
    for expected in [a, b] {
        assert!(matches!(
            set.advance(host.runtime(), &principal, &scope).unwrap(),
            Some(Progress::Performed { branch, .. }) if *branch == expected
        ));
    }
    drop(set.close().unwrap_or_else(|_| panic!("control set closes")));
    let commits = host
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits();
    home.close(&execution).unwrap().reclaim();
    commits
}

#[test]
fn two_branch_set_stopped_prefix_killed_at_each_branch_resumes_suffix_once() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let control_commits = uninterrupted_two_branch_control(execution);

    for (mode, performed_before_stop) in [("stopped-branch-0", 0_usize), ("stopped-branch-1", 1)] {
        let dir = tempfile::tempdir().unwrap();
        let loc = dir.path().join("home");
        predecessor(&loc, true);
        let end = run_child(&loc, mode);
        assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark, "{mode}");

        let home = open_home(&loc, false, execution, None);
        let summary = home
            .runtime()
            .runtime()
            .cold_adoption_readmission_summary_for_test();
        assert_eq!(summary.stopped_sets_retained, 1, "{mode}");
        let (a, b) = two_branches(&home);
        let host = home.runtime();
        let p1 = *host
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .owned_revision();
        let retire_refusal = host.retire_program_support(&execution, &p1);
        if performed_before_stop == 0 {
            assert!(
                matches!(
                    retire_refusal,
                    Err(WorthQueryProgramSupportRetirementFailure::Policy(
                        WorthQueryProgramSupportRetirementDenial::MandatoryCustody(ref inv)
                    )) if inv.mandatory_custody() >= 1
                ),
                "{mode}: stopped set must block P1 retirement via MandatoryCustody: {retire_refusal:?}"
            );
        } else {
            assert!(
                matches!(
                    retire_refusal,
                    Err(WorthQueryProgramSupportRetirementFailure::Policy(
                        WorthQueryProgramSupportRetirementDenial::CurrentBranches(ref inv)
                    )) if inv.current_branches() == 1 && inv.mandatory_custody() >= 1
                ),
                "{mode}: adopted prefix + stopped suffix must block P1 retirement: {retire_refusal:?}"
            );
        }

        let scope = request_scope();
        let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
        let mut stopped_list = host
            .with_host_advancement(|phase| {
                host.runtime()
                    .readmit_stopped_branch_set_adoptions(&phase)
                    .unwrap()
            })
            .unwrap();
        assert_eq!(stopped_list.len(), 1, "{mode}");
        let mut stopped = stopped_list.pop().unwrap();
        let prefix_commit = if performed_before_stop == 1 {
            let [Progress::RecordedPerformed { commit, .. }, Progress::RecordedNoEffect { .. }] =
                stopped.progress()
            else {
                panic!("{mode}: expected recorded prefix + stop");
            };
            let prefix = *commit;
            let widened = host
                .runtime()
                .branches()
                .program_adoption_coverage(&[a, b], std::num::NonZeroUsize::new(2).unwrap())
                .unwrap();
            let failure = host
                .runtime()
                .request(&principal, &scope)
                .on_branches(widened, &[a, b])
                .unwrap()
                .programs()
                .resume(stopped, 64)
                .err()
                .expect("widened coverage must refuse re-preparing the stopped prefix");
            assert!(matches!(
                failure.denial(),
                WorthQueryBranchSetAdoptionResumeDenial::CoverageMismatch
            ));
            stopped = failure.into_adoption();
            Some(prefix)
        } else {
            assert!(matches!(stopped.progress(), [Progress::RecordedNoEffect { .. }]));
            None
        };

        let remaining = stopped.remaining_branches().to_vec();
        assert_eq!(remaining.len(), 2 - performed_before_stop, "{mode}");
        let coverage = host
            .runtime()
            .branches()
            .program_adoption_coverage(
                &remaining,
                std::num::NonZeroUsize::new(remaining.len()).unwrap(),
            )
            .unwrap();
        let mut set = host
            .runtime()
            .request(&principal, &scope)
            .on_branches(coverage, &remaining)
            .unwrap()
            .programs()
            .resume(stopped, 64)
            .unwrap_or_else(|failure| panic!("{mode}: resume suffix: {:?}", failure.denial()));
        for expected in &remaining {
            assert!(matches!(
                set.advance(host.runtime(), &principal, &scope).unwrap(),
                Some(Progress::Performed { branch, .. }) if branch == expected
            ));
        }
        let closed = set.close().unwrap_or_else(|_| panic!("{mode}: close set"));
        assert_eq!(closed.progress().len(), 2, "{mode}");
        if let Some(prefix) = prefix_commit {
            assert!(matches!(
                &closed.progress()[0],
                Progress::RecordedPerformed { commit, .. } if *commit == prefix
            ));
        }
        drop(closed);
        // 1 extra commit is the intentional source edit that moved the branch head.
        assert_eq!(
            host.runtime()
                .world_history_snapshot_for_test()
                .installed_commits(),
            control_commits + 1,
            "{mode}: each branch adopted once; stopped prefix never re-prepared"
        );
        for (idx, branch) in [a, b].into_iter().enumerate() {
            assert_eq!(
                settle(set_retention(host, branch, 15, 0xB600 + idx as u64)),
                RetentionVerdict::Performed(15),
                "{mode}: branch {idx} enforces P1"
            );
        }
        home.close(&execution).unwrap().reclaim();
        let reopened = open_home(&loc, false, execution, None);
        assert!(reopened
            .runtime()
            .with_host_advancement(|phase| reopened
                .runtime()
                .runtime()
                .readmit_stopped_branch_set_adoptions(&phase)
                .unwrap())
            .unwrap()
            .is_empty());
        reopened.close(&execution).unwrap().reclaim();
    }
}

#[test]
fn program_support_retirement_survives_process_kill_and_refuses_interpretation() {
    let dir = tempfile::tempdir().unwrap();
    let loc = dir.path().join("home");
    predecessor(&loc, false);
    let end = run_child(&loc, "retire-and-kill");
    assert_eq!(end, crate::killed_child::ChildEnd::KilledAtPark);

    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_home(&loc, false, execution, None);
    {
        let host = home.runtime();
        let p0 = *host.owned_revision();
        assert!(matches!(
            host.retire_program_support(&execution, &p0),
            Err(WorthQueryProgramSupportRetirementFailure::Policy(
                WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision }
            )) if revision == p0
        ));
        let branch = host.current_world();
        let before = observe_head(host.runtime(), branch);
        let WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::Denied(denial),
        ) = set_retention(host, branch, SEED_RETENTION + 1, 0x9176_5331)
            .expect("retired P0 request reaches the commit gate")
        else {
            panic!("retired P0 action must be refused at the commit gate after kill");
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProgramSupportRetired
        );
        assert_eq!(read_retention(host.runtime(), branch), SEED_RETENTION);
        assert_eq!(
            before.selected_commit(),
            observe_head(host.runtime(), branch).selected_commit()
        );
    }
    home.close(&execution).unwrap().reclaim();
    let reopened = open_home(&loc, false, execution, None);
    let p0 = *reopened.runtime().owned_revision();
    assert!(matches!(
        reopened.runtime().retire_program_support(&execution, &p0),
        Err(WorthQueryProgramSupportRetirementFailure::Policy(
            WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision }
        )) if revision == p0
    ));
    reopened.close(&execution).unwrap().reclaim();
}
