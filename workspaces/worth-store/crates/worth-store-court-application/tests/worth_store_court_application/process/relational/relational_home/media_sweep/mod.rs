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

#[test]
fn fresh_reopen_media_fault_refuses_typed_without_panic_and_reopens_cleanly() {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let script = crate::workload::Script::generate(SEEDS[0], COMMITS, CHECKPOINT_EVERY);
    let target = replacement_checkpoint(&script);
    let prefix = &script.steps[..target.index()];
    let mut prepared = PreparedSweepHome::prepare(prefix, &execution);
    let schema = served_schema(prefix);
    let commits = prefix.iter().filter(|step| step.is_commit()).count();

    let fresh_reopen_stage =
        |err: &worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused| {
            let Some(worth_store_application::ApplicationHomeOpenRefusal::Physical(p)) =
                err.source_cause()
            else {
                return None;
            };
            if p.stage() != worth_store_application::ApplicationHomePhysicalStage::Recovery
                || !p.detail().contains("PublicationIndeterminate")
            {
                return None;
            }
            let (_, reopen_detail) = p
                .detail()
                .split_once("reopen: Some(PhysicalRecoveryReopenFailure {")?;
            let reopen_detail = reopen_detail.split("})").next().unwrap_or(reopen_detail);
            if reopen_detail.contains("stage: CurrentSelector") {
                Some("CurrentSelector")
            } else if reopen_detail.contains("stage: RootManifest") {
                Some("RootManifest")
            } else {
                None
            }
        };

    let assert_clean_reopen = |location: &std::path::Path, label: &str| {
        let recovered = crate::durable_run::open_declaring(location, schema, &execution)
            .unwrap_or_else(|refusal| {
                panic!("{label}: clean reopen after fault refused: {refusal:?}")
            });
        assert_eq!(
            crate::workload::states_of(recovered.relational().runtime(), &script.records),
            script.states[commits],
            "{label}: clean reopen must preserve full Home durability and state"
        );
    };

    // 1. Every identified `PositionedRead` during Home open (including fresh reopen #1/#2).
    prepared.restore();
    let counter = media_faults::FaultedMedia::admit(&[(
        MediaOperationRole::PositionedRead,
        NonZeroU64::MAX,
        media_faults::fail_before(),
    )])
    .expect("PositionedRead counter schedule admits");
    media_faults::arm(&counter.activations);
    let req = worth_store_testkit_application_oracle::facade::home_opening::request(
        prepared.location(),
        crate::workload::runtime_builder(schema),
    );
    let home = worth_store_testkit_application_oracle::facade::home_opening::open_with_media(
        req,
        counter.admission,
        &execution,
    )
    .expect("unfaulted open succeeds during measurement");
    let read_total = counter.activations[0].matching_operation_count();
    drop(home);
    assert!(
        read_total >= 2,
        "PositionedRead must execute during fresh home open"
    );
    let mut read_selector_reopen = 0u64;
    let mut read_root_reopen = 0u64;
    for nth in 1..=read_total {
        prepared.restore();
        let faulted = media_faults::FaultedMedia::admit(&[(
            MediaOperationRole::PositionedRead,
            NonZeroU64::new(nth).unwrap(),
            media_faults::fail_before(),
        )])
        .expect("PositionedRead fail-before schedule admits");
        media_faults::arm(&faulted.activations);
        let req = worth_store_testkit_application_oracle::facade::home_opening::request(
            prepared.location(),
            crate::workload::runtime_builder(schema),
        );
        let outcome = worth_store_testkit_application_oracle::facade::home_opening::open_with_media(
            req,
            faulted.admission,
            &execution,
        );
        assert!(
            faulted.activations[0].is_consumed(),
            "PositionedRead#{nth}: fault must fire during home open"
        );
        match outcome {
            Ok(home) => {
                drop(home);
                assert_clean_reopen(prepared.location(), &format!("PositionedRead#{nth}"));
            }
            Err(refusal) => {
                match fresh_reopen_stage(&refusal) {
                    Some("CurrentSelector") => read_selector_reopen += 1,
                    Some("RootManifest") => read_root_reopen += 1,
                    _ => {}
                }
                drop(refusal);
                assert_clean_reopen(prepared.location(), &format!("PositionedRead#{nth}"));
            }
        }
    }
    assert!(
        read_selector_reopen >= 1 && read_root_reopen >= 1,
        "PositionedRead sweep must hit fresh-reopen PublicationIndeterminate for both CurrentSelector ({read_selector_reopen}) and RootManifest ({read_root_reopen})"
    );

    // 2. Every raw `OpenExisting` ordinal during Home open (including fresh reopen #1/#2).
    let mut open_selector_reopen = 0u64;
    let mut open_root_reopen = 0u64;
    let mut consecutive_ok = 0u64;
    for ordinal in 1..=512u64 {
        prepared.restore();
        let admission = worth_store::physical_runtime::FilesystemMediaAdmission::certification(
            worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount,
        );
        let authority = admission.fault_schedule_authority();
        let rule = authority.rule(
            MediaOperationRole::OpenExisting,
            ordinal,
            media_faults::fail_before(),
        );
        let schedule = authority.schedule(vec![rule]).unwrap();
        let req = worth_store_testkit_application_oracle::facade::home_opening::request(
            prepared.location(),
            crate::workload::runtime_builder(schema),
        );
        let outcome = worth_store_testkit_application_oracle::facade::home_opening::open_with_media(
            req,
            admission.with_fault_schedule(schedule),
            &execution,
        );
        match outcome {
            Ok(home) => {
                drop(home);
                consecutive_ok += 1;
                if consecutive_ok >= 8 {
                    break;
                }
            }
            Err(refusal) => {
                consecutive_ok = 0;
                match fresh_reopen_stage(&refusal) {
                    Some("CurrentSelector") => open_selector_reopen += 1,
                    Some("RootManifest") => open_root_reopen += 1,
                    _ => {}
                }
                drop(refusal);
                assert_clean_reopen(prepared.location(), &format!("OpenExisting#{ordinal}"));
            }
        }
    }
    assert!(
        open_selector_reopen >= 1 && open_root_reopen >= 1,
        "OpenExisting sweep must hit fresh-reopen PublicationIndeterminate for both CurrentSelector ({open_selector_reopen}) and RootManifest ({open_root_reopen})"
    );
}
