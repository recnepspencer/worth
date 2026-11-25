//! C3.6 real process-kill court: corrections on divergent root and forked
//! branches survive OS kill without close, keep exact branch-local lineage,
//! refuse cross-branch alternatives (ForeignAlternative & TargetIncarnationChanged),
//! and reapply cleanly without moving the sibling branch.
use super::numeric_home::{
    self, find_fork_branch, fork_branch, head_ordinal, model, open_program_home, read_rows,
    DirectoryProgram as Program,
};
use super::*;
use crate::process::host::durable_retry::child_reexecution::{
    drive_test, expected, location, wait_for_kill,
};
use model::{
    authentication::*,
    edit::{Edit, Value},
    other_edit::OtherEdit,
    query::Row,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::*,
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::*,
    product::WorthQueryProductBranch,
    runtime::ExecutionAllocationPolicy,
};

const TEST_PATH: &str = "host::application_home::correction_material::kill_divergent_branch::divergent_branch_correction_killed_and_reopened_keeps_branch_lineage";
const ORIG_KEY: &str = "div-kill-orig";
const ROOT_SEC_KEY: &str = "div-kill-root-sec";
const ROOT_REV_KEY: &str = "div-kill-root-rev";
const FORK_SEC_KEY: &str = "div-kill-fork-sec";
const FORK_REV_KEY: &str = "div-kill-fork-rev";
const FORK_REAPPLY_KEY: &str = "div-kill-fork-reapply";

#[derive(serde::Serialize, serde::Deserialize)]
struct DivergentRecord {
    orig_commit: u64,
    root_rev_commit: u64,
    root_head_ordinal: u64,
    fork_rev_commit: u64,
    fork_head_ordinal: u64,
}

fn edit_primary(
    app: &WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, Program>,
    branch: WorthQueryProductBranch,
    value: u64,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let key = key.to_owned();
    let outcome = app
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(value),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("fresh primary edit {key} must commit: {other:?}"),
    }
}

fn edit_secondary(
    app: &WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, Program>,
    branch: WorthQueryProductBranch,
    value: u64,
    key: &str,
) {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let key = key.to_owned();
    let outcome = app
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value: Value::Secondary(value),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "fresh secondary edit {key} must commit: {outcome:?}"
    );
}

fn reverse_branch(
    app: &WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, Program>,
    branch: WorthQueryProductBranch,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let req = app.runtime().request(&principal, &scope).on_branch(branch);
    let selected = req
        .corrections()
        .select(source, numeric_home::limits())
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = req
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(numeric_home::limits())
        .prepare()
        .unwrap()
    else {
        panic!("branch reversal {key} must prepare");
    };
    match prepared.publish().unwrap() {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("branch reversal {key} must commit: {other:?}"),
    }
}

fn replay_coords(
    app: &WorthQueryProgramApplicationRuntime<model::schema::NumericSchema, Program>,
    branch: WorthQueryProductBranch,
    value: u64,
    key: &str,
) -> (WorthQueryCorrectionSourceCoordinates, u64) {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let key_owned = key.to_owned();
    let WorthQueryApplicationMutationOutcome::RecoveredCommitted(replayed) = app
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .advanced()
        .mutate(Edit {
            key: "numeric".to_owned(),
            value: Value::Primary(value),
        })
        .without_source()
        .idempotency(&key_owned)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("branch key {key} must replay RecoveredCommitted");
    };
    let durable = replayed.new_product_head().durable().unwrap();
    let coords = WorthQueryCorrectionSourceCoordinates::from_untrusted_parts(
        worth_foundational::facade::WorldOwnerCoordinate::Home {
            owner: durable.branch().owner(),
        },
        durable.selected_commit().ordinal(),
        replayed.commit_reference().clone(),
        worth_query_host::facade::primary_graph::WorthQueryCorrectionSourceMember::Primary,
    );
    (coords, replayed.commit_reference().commit_id.0)
}

#[test]
fn divergent_branch_correction_killed_and_reopened_keeps_branch_lineage() {
    drive_test(TEST_PATH, writer, reader);
}

fn writer() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open_program_home(location(), execution, true);
    let app = home.runtime();
    let root = app.runtime().current_world();
    let orig = edit_primary(app, root, 11, ORIG_KEY);
    let fork = fork_branch(&home);

    // Diverge root and fork with distinct disjoint Secondary values, then
    // reverse the shared pre-fork source independently on each branch.
    edit_secondary(app, root, 19, ROOT_SEC_KEY);
    let root_rev = reverse_branch(app, root, &orig, ROOT_REV_KEY);
    edit_secondary(app, fork, 29, FORK_SEC_KEY);
    let fork_rev = reverse_branch(app, fork, &orig, FORK_REV_KEY);

    assert_eq!(read_rows(app, root), [Row { operand: None, primary: 5, secondary: 19 }]);
    assert_eq!(read_rows(app, fork), [Row { operand: None, primary: 5, secondary: 29 }]);
    assert_ne!(
        root_rev.commit_reference().commit_id,
        fork_rev.commit_reference().commit_id
    );
    let record = DivergentRecord {
        orig_commit: orig.commit_reference().commit_id.0,
        root_rev_commit: root_rev.commit_reference().commit_id.0,
        root_head_ordinal: head_ordinal(app, root),
        fork_rev_commit: fork_rev.commit_reference().commit_id.0,
        fork_head_ordinal: head_ordinal(app, fork),
    };
    std::fs::write(expected(), serde_json::to_vec(&record).unwrap()).unwrap();
    // Kill without closing Home!
    wait_for_kill();
}

fn reader() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let record: DivergentRecord = serde_json::from_slice(&std::fs::read(expected()).unwrap()).unwrap();
    let home = open_program_home(location(), execution, false);
    {
        let app = home.runtime();
        let root = app.runtime().current_world();
        let fork = find_fork_branch(&home);
        assert_eq!(head_ordinal(app, root), record.root_head_ordinal);
        assert_eq!(head_ordinal(app, fork), record.fork_head_ordinal);
        assert_eq!(read_rows(app, root), [Row { operand: None, primary: 5, secondary: 19 }]);
        assert_eq!(read_rows(app, fork), [Row { operand: None, primary: 5, secondary: 29 }]);

        let (orig_coords, orig_id) = replay_coords(app, root, 11, ORIG_KEY);
        assert_eq!(orig_id, record.orig_commit);

        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let root_req = app.runtime().request(&principal, &scope).on_branch(root);
        let fork_req = app.runtime().request(&principal, &scope).on_branch(fork);

        // Each branch replays its own reversal of the shared source without new commits.
        let root_src = root_req
            .corrections()
            .select_coordinates(&orig_coords, numeric_home::limits())
            .unwrap();
        let WorthQueryApplicationCorrectionPreparation::Settled(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(root_rev_replay),
        ) = root_req
            .corrections()
            .reverse(&root_src)
            .idempotency(ROOT_REV_KEY)
            .limits(numeric_home::limits())
            .prepare()
            .unwrap()
        else {
            panic!("root reversal must replay RecoveredCommitted");
        };
        assert_eq!(root_rev_replay.commit_reference().commit_id.0, record.root_rev_commit);
        let root_causality = root_rev_replay.aftermath_causality().expect("root reversal causality");
        assert_eq!(root_causality.role(), WorthQueryAftermathCausalRole::RecordedReverse);
        assert_eq!(root_causality.source().relational().commit_id.0, record.orig_commit);

        let fork_src = fork_req
            .corrections()
            .select_coordinates(&orig_coords, numeric_home::limits())
            .unwrap();
        let WorthQueryApplicationCorrectionPreparation::Settled(
            WorthQueryApplicationMutationOutcome::RecoveredCommitted(fork_rev_replay),
        ) = fork_req
            .corrections()
            .reverse(&fork_src)
            .idempotency(FORK_REV_KEY)
            .limits(numeric_home::limits())
            .prepare()
            .unwrap()
        else {
            panic!("fork reversal must replay RecoveredCommitted");
        };
        assert_eq!(fork_rev_replay.commit_reference().commit_id.0, record.fork_rev_commit);
        let fork_causality = fork_rev_replay.aftermath_causality().expect("fork reversal causality");
        assert_eq!(fork_causality.role(), WorthQueryAftermathCausalRole::RecordedReverse);
        assert_eq!(fork_causality.source().relational().commit_id.0, record.orig_commit);

        // Each branch's alternatives page carries only its own reversal; selecting
        // fork's metadata on root refuses ForeignAlternative and reapplying fork's
        // selected carrier on root refuses TargetIncarnationChanged.
        let root_page = root_req
            .corrections()
            .alternatives(&root_src, numeric_home::limits(), NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(root_page.entries().len(), 1);
        assert_eq!(root_page.entries()[0].performed_commit().commit_id.0, record.root_rev_commit);

        let fork_page = fork_req
            .corrections()
            .alternatives(&fork_src, numeric_home::limits(), NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert_eq!(fork_page.entries().len(), 1);
        assert_eq!(fork_page.entries()[0].performed_commit().commit_id.0, record.fork_rev_commit);

        let foreign = root_page
            .select(&fork_page.entries()[0], numeric_home::limits())
            .err()
            .expect("root page must refuse fork's alternative metadata");
        assert!(
            matches!(
                &foreign,
                WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::ForeignAlternative
                )
            ),
            "expected ForeignAlternative, got {foreign:?}"
        );
        let fork_alt = fork_page
            .select(&fork_page.entries()[0], numeric_home::limits())
            .unwrap();
        let retargeted = root_req
            .corrections()
            .reapply(&fork_alt)
            .idempotency("cross-branch-forbidden")
            .limits(numeric_home::limits())
            .prepare()
            .err()
            .expect("root branch must refuse fork's selected carrier");
        assert!(
            matches!(
                &retargeted,
                WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::TargetIncarnationChanged
                )
            ),
            "expected TargetIncarnationChanged, got {retargeted:?}"
        );
        assert_eq!(head_ordinal(app, root), record.root_head_ordinal);

        // Reapplying fork's alternative on fork restores Primary(11) while keeping
        // fork's Secondary(29) and leaving root untouched at (5, 19).
        let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = fork_req
            .corrections()
            .reapply(&fork_alt)
            .idempotency(FORK_REAPPLY_KEY)
            .limits(numeric_home::limits())
            .prepare()
            .unwrap()
        else {
            panic!("fork reapply must prepare");
        };
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } =
            prepared.publish().unwrap()
        else {
            panic!("fork reapply must commit");
        };
        let fact = receipt.aftermath_causality().expect("reapply lineage");
        assert_eq!(fact.role(), WorthQueryAftermathCausalRole::RecordedReapply);
        assert_eq!(fact.recorded_source().relational().commit_id.0, record.orig_commit);
        assert_eq!(read_rows(app, fork), [Row { operand: None, primary: 11, secondary: 29 }]);
        assert_eq!(read_rows(app, root), [Row { operand: None, primary: 5, secondary: 19 }]);
        assert_eq!(head_ordinal(app, root), record.root_head_ordinal);
    }
    home.close(&execution)
        .unwrap_or_else(|e| panic!("first divergent reopen close: {e:?}"))
        .reclaim();

    // Second reopen confirms both branches retain their distinct lineages.
    let home2 = open_program_home(location(), execution, false);
    let app = home2.runtime();
    let root = app.runtime().current_world();
    let fork = find_fork_branch(&home2);
    assert_eq!(read_rows(app, root), [Row { operand: None, primary: 5, secondary: 19 }]);
    assert_eq!(read_rows(app, fork), [Row { operand: None, primary: 11, secondary: 29 }]);
    assert_eq!(head_ordinal(app, root), record.root_head_ordinal);
    home2
        .close(&execution)
        .unwrap_or_else(|e| panic!("second divergent reopen close: {e:?}"))
        .reclaim();
}
