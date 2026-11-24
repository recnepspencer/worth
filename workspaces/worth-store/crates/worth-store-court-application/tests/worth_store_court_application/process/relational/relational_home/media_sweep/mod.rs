//! Media sweep: every identified media operation of a home commit and of a
//! home checkpoint, faulted once each with `FailBefore` and
//! `IndeterminateAfterEffect`.
//!
//! For each seed and target the sweep counts every role's operations while
//! the target runs on a home holding the script before it, then runs one
//! case per (role, ordinal, directive): the in-flight commit is recovered
//! exactly when the journal holds it, and then it is the artifact its
//! unresolved fate names; a proven-no-effect commit is absent; a checkpoint
//! fault leaves the old or the new checkpoint selected; the rest of the
//! script, a second recovery and fresh identities follow once per distinct
//! post-recovery equivalence class.
//!
//! Targets: the first commit, each of fork/archive/deletion-requested/delete,
//! the first commit on a durably forked branch, the schema-transition commit,
//! and a checkpoint that replaces a selected one; two seeds each.

#[allow(unused_imports)]
use crate::media_sweep::*;

use crate::workload::Step;

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
    sweep(|script| {
        crate::media_case::Target::BranchTransition(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::Archive { .. }))
                .unwrap(),
        )
    });
    sweep(|script| {
        crate::media_case::Target::BranchTransition(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::RequestDeletion { .. }))
                .unwrap(),
        )
    });
    sweep(|script| {
        crate::media_case::Target::BranchTransition(
            script
                .steps
                .iter()
                .position(|step| matches!(step, Step::Delete { .. }))
                .unwrap(),
        )
    });
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

#[test]
#[ignore = "blocked by worth-relational checkpoint_complete::metadata::members::dictionary_pages::capture dropping prior cold symbol pages; un-ignore and remove the state_of warm-up in prepare_world_checkpoint_home once fixed"]
fn cold_symbol_page_checkpoint_replacement_recovers_without_prewarm() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let script = crate::workload::Script::generate(SEEDS[0], COMMITS, CHECKPOINT_EVERY);
    let target = replacement_checkpoint(&script);
    let prefix = &script.steps[..target.index()];
    let prepared = crate::prepared_home::PreparedHome::prepare(prefix, &execution);
    let schema = served_schema(prefix);

    let fixture = crate::durable_run::world::open(prepared.location(), schema, false, &execution)
        .expect("cold checkpointed World home opens");
    assert_eq!(
        fixture.home.relational().cold_storage_reads().member_reads,
        0,
        "prior symbol and region pages must remain cold before replacement checkpoint"
    );
    crate::media_case::drain_world_clean_frames(&fixture);
    fixture
        .home
        .checkpoint(&execution)
        .expect("cold replacement checkpoint succeeds");
    drop(fixture);

    let recovered = crate::durable_run::open_declaring(prepared.location(), schema, &execution)
        .expect("recovered home after cold replacement checkpoint opens");
    let commits = prefix.iter().filter(|step| step.is_commit()).count();
    assert_eq!(
        crate::workload::states_of(recovered.relational().runtime(), &script.records),
        script.states[commits],
        "cold replacement checkpoint must preserve prior cold symbol pages"
    );
}
