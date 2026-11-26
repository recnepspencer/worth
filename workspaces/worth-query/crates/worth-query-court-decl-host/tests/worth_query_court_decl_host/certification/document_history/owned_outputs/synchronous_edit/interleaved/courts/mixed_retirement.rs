//! Public #348: a source-scope retirement prefix beside later authored edits in
//! one envelope commits exactly the world of its stages committed one at a time.
//! Only the prefix retires until per-stage retirement events exist (#369).
//! A restoration's own reconciliation routes beside later stages are #371.
use super::super::super::super::authored_member as member;
use super::super::super::courts::native_head;
use super::*;
use program::CompletionRetirementProgram as RetirementProgram;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryEditEnvelopeReceipt as Receipt, WorthQueryRuntimeEditEnvelope as Envelope,
};
use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;
type RetirementHost = WorthQueryProgramApplicationRuntime<Schema, RetirementProgram>;
/// `a`'s value, its owned output's value, and whether `m` and its owned output are live.
type World = (u64, u64, bool, bool);

#[derive(Clone, Copy, Debug)]
enum Stage {
    /// The installed authored member retirement (`m` and its owned output).
    Retire,
    /// An edit of model `a` with no output graph; `a` owns a current output.
    Plain,
    /// An edit whose declared output graph republishes `a`'s owned route.
    Producing,
    /// An edit of `a` its handler refuses (`value > 1000`).
    Refused,
    /// An edit of model `a` whose declared output graph re-derives `a`'s owned route.
    Rederiving,
}

fn configuration() -> world::Configuration {
    world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() }
}

fn limits() -> WorthQueryApplicationLimits {
    host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}

/// A fresh host where `a` and `m` both own settled source-scope outputs.
fn settled() -> RetirementHost {
    let host: RetirementHost = world::install_configured(limits(), 1, None, configuration());
    settle(&host);
    host
}

/// The same settled world on a Directory Home that can close and reopen.
fn settled_directory() -> world::DirHost<RetirementProgram> {
    let host = world::open_directory_configured::<RetirementProgram>(limits(), 1, None, configuration());
    settle(&host);
    host
}

fn settle(host: &RetirementHost) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    macro_rules! settle {
        ($demand:expr, $tree:ty) => {{
            let mut operation = request
                .demand($demand)
                .retain_program_outputs::<RetirementProgram, $tree>(&host)
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
            for count in 0..128 {
                assert!(count < 127, "owned output did not settle");
                operation = match operation.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                    WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                        done.deliver_current(&request).unwrap();
                        done.close(&request).unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                        break;
                    }
                    WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
                    WorthQueryProgramOutputOperationAdvance::Terminal(_) => panic!("owned output terminated"),
                };
            }
        }};
    }
    settle!(first::Demand("a".into()), ModelTree);
    settle!(downstream::Demand("m".into()), program::OwnedMemberTree);
}

fn push(edit: &mut Envelope<'_, '_, Schema, RetirementProgram, String>, stage: Stage) {
    let pushed = match stage {
        Stage::Retire => edit.push_stage::<ApplicationNoOutputGraph, _>(member::retire::Intent).map(drop),
        Stage::Plain | Stage::Refused => edit
            .push_stage::<ApplicationNoOutputGraph, _>(fixture::cyclic::Intent {
                key: "a".into(),
                value: if matches!(stage, Stage::Refused) { 1001 } else { 3 },
                key_derived_target: false,
            })
            .map(drop),
        Stage::Rederiving => edit
            .push_stage::<ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 5,
                key_derived_target: false,
            })
            .map(drop),
        Stage::Producing => edit
            .push_stage::<ModelTree, _>(fixture::cap::Intent {
                key: "envelope-cap".into(),
                value: 8,
                key_derived_target: false,
            })
            .map(drop),
    };
    pushed.unwrap();
}

/// One ordered authored envelope of `stages` through the public runtime entry.
fn try_run(
    host: &RetirementHost,
    key: &str,
    stages: &[Stage],
) -> Result<Outcome, impl std::fmt::Debug> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency(key.to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    for stage in stages {
        push(&mut edit, *stage);
    }
    edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
}

fn run(host: &RetirementHost, key: &str, stages: &[Stage]) -> Outcome {
    try_run(host, key, stages).unwrap()
}

/// The correction inverse of `retired`, then `stages`, as one envelope. The
/// undo must declare `ModelTree`: its reconciliation re-derives `first:a`.
fn undo(host: &RetirementHost, retired: &Receipt, key: &str, stages: &[Stage]) -> Outcome {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request.corrections().select(retired.receipt(), limits).unwrap();
    let mut inverse = request
        .corrections()
        .reverse(&source)
        .idempotency(key)
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, RetirementProgram, ModelTree>(host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    for stage in stages {
        push(&mut inverse, *stage);
    }
    inverse.execute_performed(ExecutionAllocationPolicy::SystemAllocation).unwrap()
}

fn observe(host: &RetirementHost) -> World {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    (
        request.query(query::model::Read("a".into())).execute().unwrap().rows()[0].value,
        request.query(query::first_output::Read("first:a".into())).execute().unwrap().rows()[0].value,
        request.query(query::member_public::Read("m".into())).execute().is_ok(),
        request.query(query::child_output::Read("downstream:m".into())).execute().is_ok(),
    )
}

#[track_caller]
fn committed(outcome: Outcome, stages: usize) -> Receipt {
    let Outcome::Committed(receipt) = outcome else { panic!("{outcome:?}") };
    assert_eq!(receipt.stage_count(), stages);
    receipt
}

/// Each stage alone and in order: the oracle, and the controls that each commit.
fn sequential(stages: &[Stage]) -> World {
    let host = settled();
    for (ordinal, stage) in stages.iter().enumerate() {
        committed(run(&host, &format!("alone-{ordinal}"), &[*stage]), 1);
    }
    observe(&host)
}

#[test]
fn a_retirement_prefix_beside_later_edits_commits_its_stages_in_order() {
    for stages in [&[Stage::Retire, Stage::Plain][..], &[Stage::Retire, Stage::Plain, Stage::Producing]] {
        let expected = sequential(stages);
        assert!(!expected.2 && !expected.3, "the oracle retires m and its owned output");
        let host = settled();
        let before = native_head(&host).0;
        committed(run(&host, "mixed", stages), stages.len());
        assert_eq!(native_head(&host).1, [before], "{stages:?} is one commit");
        assert_eq!(observe(&host), expected, "{stages:?} equals its stages one at a time");
    }
    // Directory Home intentionally refuses relation-only inverse preimages
    // (`owned-outputs.authored-member-retire.v1`), matching `borrowed_inputs`
    // and `member_retarget`. Verify the exact refusal guard on reopenable Home.
    let dir_host = settled_directory();
    let err = try_run(&dir_host, "mixed-dir", &[Stage::Retire, Stage::Plain])
        .expect_err("Directory Home refuses relation-only inverse preimage");
    assert!(
        format!("{err:?}").contains("UnsupportedCorrectionRelationPreimage"),
        "{err:?}"
    );
}

/// The undo of the retirement and `stages` as one envelope commit once, equal
/// to the undo and each stage committed alone.
fn restoration_equals_sequential(stages: &[Stage]) {
    let expected = {
        let host = settled();
        let retired = committed(run(&host, "retire", &[Stage::Retire]), 1);
        committed(undo(&host, &retired, "undo", &[]), 1);
        for (ordinal, stage) in stages.iter().enumerate() {
            committed(run(&host, &format!("alone-{ordinal}"), &[*stage]), 1);
        }
        observe(&host)
    };
    assert!(expected.2 && expected.3, "the oracle restores m and its owned output");
    let host = settled();
    let retired = committed(run(&host, "retire", &[Stage::Retire]), 1);
    let before = native_head(&host).0;
    committed(undo(&host, &retired, "undo-and-edit", stages), stages.len() + 1);
    assert_eq!(native_head(&host).1, [before], "{stages:?} is one commit");
    assert_eq!(observe(&host), expected, "{stages:?} equals its stages one at a time");
    let dir_host = settled_directory();
    let err = try_run(&dir_host, "retire-dir", &[Stage::Retire])
        .expect_err("Directory Home refuses relation-only inverse preimage");
    assert!(
        format!("{err:?}").contains("UnsupportedCorrectionRelationPreimage"),
        "{err:?}"
    );
}

#[test]
fn a_restoration_prefix_beside_later_edits_commits_its_stages_in_order() {
    // The retirement is completion-only, so its restoration has no recorded
    // coupled group; its later stages still run, never a front-only commit.
    restoration_equals_sequential(&[Stage::Rederiving]);
    restoration_equals_sequential(&[Stage::Rederiving, Stage::Producing]);
}

#[test]
#[ignore = "held on public #371"]
fn a_restoration_prefix_beside_a_later_no_graph_edit_of_its_reconciled_source_commits() {
    // Refuses CoupledReadSupportChanged today: the undo's reconciliation
    // derives `first:a` before the stage edits `a`, and nothing re-derives it.
    restoration_equals_sequential(&[Stage::Plain]);
}

#[test]
#[ignore = "held on public #371"]
fn a_restoration_prefix_beside_a_later_republication_of_its_reconciled_route_commits() {
    // Refuses ExtraCoupledOwner (catalog-route-absent) today: the restoration
    // custody without a recorded group lists no reconciliation route.
    restoration_equals_sequential(&[Stage::Producing]);
}

#[test]
fn a_refused_later_stage_commits_no_retirement_or_restoration_prefix() {
    // Atomicity: the prefix never commits without its later stages.
    for restore in [false, true] {
        let host = settled();
        let retired = restore.then(|| committed(run(&host, "retire", &[Stage::Retire]), 1));
        let (before, seen) = (native_head(&host), observe(&host));
        let (outcome, writes) = observe_maintenance_work(24, || match &retired {
            Some(retired) => undo(&host, retired, "undo-refused", &[Stage::Refused]),
            None => run(&host, "retire-refused", &[Stage::Retire, Stage::Refused]),
        });
        let Outcome::DomainDenied(denial) = &outcome else { panic!("{outcome:?}") };
        assert_eq!(denial.identity().ordinal(), 1, "the later stage owns the refusal");
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(native_head(&host), before, "no prefix commit");
        assert_eq!(observe(&host), seen);
    }
}

#[test]
fn a_retirement_after_the_prefix_refuses_typed_before_any_effect() {
    // Held capability: only the prefix retires until public #369.
    let host = settled();
    let (before, seen) = (native_head(&host), observe(&host));
    let (outcome, writes) =
        observe_maintenance_work(24, || run(&host, "late-retire", &[Stage::Plain, Stage::Retire]));
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = &outcome else {
        panic!("{outcome:?}")
    };
    let refusal = cause.authored_retirement_error().and_then(|error| error.authored_retirement_refusal());
    assert!(
        matches!(refusal, Some(Cause::LaterStageRetirementPostponed { issue: 369, stage: 1, owner: _ })),
        "{cause:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(native_head(&host), before, "nothing published");
    assert_eq!(observe(&host), seen);
}
