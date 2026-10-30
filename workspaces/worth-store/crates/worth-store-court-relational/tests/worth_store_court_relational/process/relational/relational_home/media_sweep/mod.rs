//! Media sweep: every identified media operation of a home commit and of a
//! home checkpoint, faulted once each with `FailBefore` and
//! `IndeterminateAfterEffect`.
//!
//! For each seed and target the sweep counts every role's operations while
//! the target runs on a home holding the script before it, then runs one
//! case per (role, ordinal, directive) (`media_case`): the in-flight commit
//! is recovered exactly when the journal holds it, and then it is the
//! artifact its unresolved fate names; a proven-no-effect commit is absent; a
//! checkpoint fault leaves the old or the new checkpoint selected; the rest
//! of the script, a second recovery and fresh identities follow.
//!
//! Targets: the first commit, each of fork/archive/deletion-requested/delete,
//! the first commit on a durably forked branch, the schema-transition commit,
//! and a checkpoint that replaces a selected one; two seeds each.

#[allow(unused_imports)]
use crate::media_sweep::*;
use std::num::NonZeroU64;
use std::sync::Mutex;
use {std::panic, std::panic::AssertUnwindSafe};
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};

use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::certification::MediaOperationRole;

use crate::media_faults;
use {crate::durable_run::apply, crate::durable_run::served_schema};
use {crate::media_case::run_case, crate::media_case::Ended, crate::media_case::Target};
use {crate::workload, crate::workload::Script, crate::workload::Step};

use crate::media_sweep::prepared_sweep_home::PreparedSweepHome;

#[test]
fn every_media_operation_of_the_first_commit_recovers() {
    sweep(first_commit);
}

#[test]
fn every_media_operation_of_the_first_commit_on_a_fork_recovers() {
    sweep(first_feature_commit);
}

#[test]
fn every_media_operation_of_a_schema_transition_commit_recovers() {
    sweep(transition_commit);
}

#[test]
fn every_media_operation_of_a_checkpoint_replacement_recovers() {
    sweep(replacement_checkpoint);
}

#[test]
fn every_media_operation_of_a_branch_transition_recovers() {
    sweep(|script| {
        crate::media_case::Target::BranchTransition(
            script
                .steps
                .iter()
                .position(|step| matches!(step, crate::workload::Step::Fork { .. }))
                .unwrap(),
        )
    });
}

#[test]
fn every_media_operation_of_archive_and_delete_transitions_recovers() {
    for select in [
        (|step: &Step| matches!(step, crate::workload::Step::Archive { .. })) as fn(&Step) -> bool,
        |step| matches!(step, crate::workload::Step::RequestDeletion { .. }),
        |step| matches!(step, crate::workload::Step::Delete { .. }),
    ] {
        sweep(|script| {
            crate::media_case::Target::BranchTransition(
                script.steps.iter().position(select).unwrap(),
            )
        });
    }
}

#[test]
fn every_media_operation_of_the_first_checkpoint_recovers() {
    sweep(|script| {
        crate::media_case::Target::Checkpoint(
            script
                .steps
                .iter()
                .position(|step| matches!(step, crate::workload::Step::Checkpoint))
                .unwrap(),
        )
    });
}

#[test]
fn every_media_operation_of_a_cold_region_load_recovers() {
    sweep(|script| {
        crate::media_case::Target::RegionLoad(
            script
                .steps
                .iter()
                .position(|step| matches!(step, crate::workload::Step::Checkpoint))
                .unwrap()
                + 1,
        )
    });
}
