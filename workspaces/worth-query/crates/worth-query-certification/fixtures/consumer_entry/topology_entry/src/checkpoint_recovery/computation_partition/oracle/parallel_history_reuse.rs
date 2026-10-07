//! Actual retained requests judged by declared values and full compute charges.
use super::super::{entry_edit::EntryFact, facts::RegionFault};
use super::*;
use crate::checkpoint_recovery::parallel_history::reuse_cases::ReuseFacts;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;
use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionStop, WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRun as Run,
};
mod computation;
mod declaration;
mod execution;
mod seeded_world;
use execution::{adjust, demand, edit, install};
use seeded_world::{facts, seed, Change};

fn only_run(
    (contacts, mut runs, mut reports): (usize, Vec<OracleRun>, Vec<Option<u64>>),
) -> (OracleRun, Option<u64>) {
    assert_eq!(contacts, 1, "exactly one producer execution in this demand");
    assert_eq!(runs.len(), 1, "exactly one observed producer run");
    assert_eq!(
        reports.len(),
        runs[0].runs.len(),
        "one report slot per completed computation"
    );
    (runs.pop().unwrap(), reports.pop().flatten())
}
fn fresh(model: &ReuseFacts) -> (OracleRun, Option<u64>) {
    let application = install(|graph| seed(model, graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    if model.odd {
        adjust(&request, &application, ODD_Y, 1);
    }
    only_run(demand(&request, &application))
}
fn case(change: Change) {
    let _guard = checkpoint_recovery_test_guard();
    let mut model = facts();
    let application = install(|graph| seed(&model, graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before_value = model.expected_value();
    let before_work = declaration::computed_work(&model);
    let (first, first_work) = only_run(demand(&request, &application));
    assert_eq!(first.outcome.unwrap().0, before_value.unwrap());
    assert_eq!(
        first_work,
        Some(before_work),
        "declared full compute and tree work"
    );
    if let Some(edits) = change.apply(&mut model) {
        edit(&request, &application, edits, 1);
    } else {
        adjust(&request, &application, ODD_Y, 1);
    }
    let declared_work = declaration::computed_work(&model);
    assert_ne!(
        model.expected_value(),
        before_value,
        "each varied input changes the modeled outcome"
    );
    if matches!(
        change,
        Change::Join | Change::Leave | Change::DigestSwap | Change::Replacement
    ) {
        assert_ne!(
            declared_work, before_work,
            "membership and item-digest edits change modeled compute work"
        );
    }
    let (retained, retained_work) = only_run(demand(&request, &application));
    let (reference, fresh_work) = fresh(&model);
    for (label, run, charge) in [
        ("retained", &retained, retained_work),
        ("fresh", &reference, fresh_work),
    ] {
        if let Some(bits) = model.expected_value() {
            assert_eq!(
                run.outcome.as_ref().unwrap().0,
                bits,
                "{label}: declared operation value model"
            );
            if let Some(charge) = charge {
                assert_eq!(
                    charge, declared_work,
                    "{label}: declared compute-charge model"
                );
            } else {
                assert_eq!(
                    run.runs,
                    [Run::Incremental],
                    "only incremental runs omit the execution report"
                );
            }
        }
    }
    if model.expected_value().is_none() {
        let item = model.members().find(|item| item.fault).unwrap();
        let expected =
            application_computation_partition_identity(&RegionKey(item.key), &mut |_| {
                Ok::<_, ()>(())
            })
            .unwrap()
            .partition();
        assert!(
            matches!(&retained.outcome, Err(WorthQueryPartitionedComputationDenial::Partition {
            partition, cause: WorthQueryComputationPartitionStop::Owner(key)
        }) if *partition == expected && *key == item.key),
            "declared fault has its canonical partition and owner cause"
        );
    }
    assert_eq!(
        retained.outcome.as_ref().map(|(bits, _)| bits),
        reference.outcome.as_ref().map(|(bits, _)| bits),
        "reuse-on and reuse-off keep the outcome"
    );
    if !matches!(change, Change::Fault) {
        let expected = if matches!(change, Change::Input) {
            Run::Full(Cause::InputChanged)
        } else {
            Run::Incremental
        };
        assert_eq!(retained.runs, [expected], "sensitive run kind");
    }
    if matches!(change, Change::Gather) {
        assert_eq!(
            retained.calls.kernels, 1,
            "only the changed partition is recomputed"
        );
    }
    if matches!(change, Change::Shared) {
        assert_eq!(
            retained.calls.kernels, 2,
            "every partition sharing the moved fact is recomputed"
        );
    }
}
#[test]
fn gather_facts() {
    case(Change::Gather)
}
#[test]
fn shared_facts() {
    case(Change::Shared)
}
#[test]
fn key_facts() {
    case(Change::Key)
}
#[test]
fn canonical_input() {
    case(Change::Input)
}
#[test]
fn faults() {
    case(Change::Fault)
}
#[test]
fn membership_join() {
    case(Change::Join)
}
#[test]
fn membership_leave() {
    case(Change::Leave)
}
#[test]
fn persisting_item_digest_swap() {
    case(Change::DigestSwap)
}
#[test]
fn membership_replacement() {
    case(Change::Replacement)
}

#[test]
fn authority_encoding_boundary_keeps_declared_compute_charge() {
    let _guard = checkpoint_recovery_test_guard();
    let model = facts();
    // 128 legitimate bootstrap issuances force the runtime's next authority
    // past the one-byte boundary even if earlier tests already advanced it.
    let application =
        execution::install_after_authority_issuances(|graph| seed(&model, graph), 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let expected = declaration::computed_work(&model);
    let (actual, charged) = only_run(demand(&request, &application));
    assert_eq!(actual.outcome.unwrap().0, model.expected_value().unwrap());
    assert_eq!(
        charged,
        Some(expected),
        "declared compute and tree charge is independent of authority encoding width"
    );
}
