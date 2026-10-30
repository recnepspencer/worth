//! Real processes: a child opens a home, runs a script prefix and is killed
//! idle on either side of the first checkpoint, or while a commit, branch
//! transition, or checkpoint is parked at a Store mutation seam. The parent
//! recovers through production `open` and holds the home to the crash oracle.
//!
//! A commit parked before its WAL append recovers without it. One parked at
//! any later seam recovers with it, although the dead process never
//! installed it: the child observes its own live history while parked, and
//! the commit is not there (durable before live, D2).

#[allow(unused_imports)]
use crate::process_kill::*;

use {crate::durable_run::apply, crate::durable_run::open};

use crate::process_kill::recovery_court::assert_killed_home_recovers;

#[test]
fn a_home_killed_on_either_side_of_its_first_checkpoint_recovers() {
    assert_killed_home_recovers(crate::process_kill::Stop::BeforeFirstCheckpoint);
    assert_killed_home_recovers(crate::process_kill::Stop::AboveCheckpoint);
}

#[test]
fn a_commit_killed_at_each_mutation_seam_recovers_durable_before_live() {
    let _: Vec<()> =
        worth_store_test_support::sweep::run(IntoIterator::into_iter(SEAMS).collect(), |seam| {
            assert_killed_home_recovers(crate::process_kill::Stop::Commit(seam));
        });
}

#[test]
fn a_checkpoint_killed_at_each_mutation_seam_keeps_one_selected() {
    let _: Vec<()> =
        worth_store_test_support::sweep::run(IntoIterator::into_iter(SEAMS).collect(), |seam| {
            assert_killed_home_recovers(crate::process_kill::Stop::CheckpointMembers(seam));
            assert_killed_home_recovers(crate::process_kill::Stop::Checkpoint(seam));
        });
}

#[test]
fn a_branch_transition_killed_at_each_mutation_seam_recovers() {
    let _: Vec<()> =
        worth_store_test_support::sweep::run(IntoIterator::into_iter(SEAMS).collect(), |seam| {
            for stop in [
                crate::process_kill::Stop::BranchTransition(seam),
                crate::process_kill::Stop::Archive(seam),
                crate::process_kill::Stop::RequestDeletion(seam),
                crate::process_kill::Stop::Delete(seam),
            ] {
                assert_killed_home_recovers(stop);
            }
        });
}

#[test]
fn a_late_mutation_seam_is_not_mistaken_for_a_finished_child() {
    let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(&directory.path().join("home"), &execution).unwrap();
    let script = script();
    let (completed, parked) = crate::process_kill::Stop::Commit(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement).plan(&script);
    apply(&home, &execution, &script.steps[..completed]);
    std::thread::scope(|scope| {
        let gate = home
            .certification_serving()
            .certification_pause_physical_mutation_at(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement);
        let worker = scope.spawn(|| {
            // Force an actual arrival beyond the gate's first polling window.
            std::thread::sleep(std::time::Duration::from_secs(6));
            let request = crate::workload::serial_request();
            let execution = worth_execution::ExecutionRequest::serial(&request);
            script.steps[parked.unwrap()]
                .commit(home.runtime(), &execution)
                .map(|_| ())
                .map_err(|stop| format!("{stop:?}"))
        });
        let arrived = await_mutation_seam(&gate, &worker, deadline);
        gate.release();
        let result = worker.join().unwrap();
        assert!(arrived, "late Store seam was missed: {result:?}");
        result.unwrap();
    });
}

#[test]
fn carried_members_survive_a_kill_at_each_manifest_mutation_seam() {
    let _: Vec<()> =
        worth_store_test_support::sweep::run(IntoIterator::into_iter(SEAMS).collect(), |seam| {
            assert_killed_home_recovers(crate::process_kill::Stop::CheckpointCarried(seam));
        });
}

#[test]
fn acknowledged_locators_survive_each_physical_checkpoint_seam_and_return() {
    let _: Vec<()> = worth_store_test_support::sweep::run(
        IntoIterator::into_iter(PHYSICAL_STEPS).collect(),
        |step| {
            assert_killed_home_recovers(crate::process_kill::Stop::CheckpointAcknowledged(step));
        },
    );
    assert_killed_home_recovers(crate::process_kill::Stop::CheckpointReturned);
}
#[path = "child/mod.rs"]
pub(crate) mod child;
