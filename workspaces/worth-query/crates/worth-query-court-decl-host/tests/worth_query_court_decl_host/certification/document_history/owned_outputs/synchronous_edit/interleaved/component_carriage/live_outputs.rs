//! #233: correction redo's live-output check (`live_outputs.rs:49`) answers the
//! same eager and sequential. It reads the generations the active undo wrote; a
//! later newly linked consumer, derived eagerly in one positioned edit or left
//! underived by one commit per stage, does not change that answer.
use super::*;
use crate::certification::document_history::owned_outputs::authored_member as member;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationCorrectionDenial as Denial,
        WorthQueryCorrectionHistoryLimits as Limits, WorthQueryEditEnvelopeOutcome as Outcome,
    },
    runtime::ExecutionAllocationPolicy,
};

type Host = WorthQueryProgramApplicationRuntime<Schema, Program>;

fn cap(value: u64) -> fixture::cap::Intent {
    fixture::cap::Intent {
        key: "envelope-cap".into(),
        value,
        key_derived_target: false,
    }
}

fn committed<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) -> WorthQueryApplicationCommitReceipt {
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
}

/// The redo answer after: retire `m`, undo it (the undo writes a fresh `m`),
/// then link `carriage:late` to `a` either in one positioned edit (`eager`) or
/// one commit per stage.
fn redo_after_link(eager: bool) -> String {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, interleaved_records: true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let limits = Limits::new(None, NonZeroUsize::MIN);
    let retired = committed(
        request
            .advanced()
            .mutate(member::retire::Intent)
            .without_source()
            .idempotency(&"live-outputs-retire".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let selected = request.corrections().select(&retired, limits).unwrap();
    let mut undo = request
        .corrections()
        .reverse(&selected)
        .idempotency("live-outputs-undo")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, Program, ApplicationNoOutputGraph>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    undo.push_stage::<ApplicationNoOutputGraph, _>(cap(8))
        .unwrap();
    let outcome = undo
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(outcome, Outcome::Committed(_)),
        "undo: {outcome:?}"
    );
    let stages: Vec<Vec<bool>> = if eager {
        vec![vec![false, true]]
    } else {
        vec![vec![false], vec![true]]
    };
    for (ordinal, batch) in stages.into_iter().enumerate() {
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("live-outputs-link:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        for attach in batch {
            if attach {
                edit.push_stage::<ApplicationNoOutputGraph, _>(attach::Intent {
                    parent: "a".into(),
                    child: "carriage:late".into(),
                    pending_child: false,
                })
            } else {
                edit.push_stage::<Tree, _>(fixture::model::Intent {
                    key: "a".into(),
                    value: 3,
                    key_derived_target: false,
                })
            }
            .unwrap();
        }
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    }
    let selected = request.corrections().select(&retired, limits).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let answer = match request
        .corrections()
        .reapply(&alternative)
        .idempotency("live-outputs-redo")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, Program, ApplicationNoOutputGraph>(&host)
    {
        Ok(redo) => {
            let mut redo = redo.ordered_authored().unwrap();
            redo.push_stage::<ApplicationNoOutputGraph, _>(cap(9))
                .unwrap();
            match redo.execute_performed(ExecutionAllocationPolicy::SystemAllocation) {
                Ok(Outcome::Committed(_)) => "committed".to_owned(),
                other => format!("{other:?}"),
            }
        }
        Err(Denial::Derivation(cause)) => format!("refused {cause:?}"),
        Err(other) => format!("refused {other:?}"),
    };
    eprintln!("[233diff] live_outputs redo eager={eager}: {answer}");
    answer
}

#[test]
fn correction_redo_answers_the_same_after_an_eager_or_sequential_later_link() {
    let eager = redo_after_link(true);
    let sequential = redo_after_link(false);
    assert_eq!(
        eager, sequential,
        "live_outputs.rs:49 tells eager from sequential"
    );
    assert_eq!(
        eager, "committed",
        "the undo's own fresh member is live in both worlds"
    );
}
