//! Actual retained requests judged by declared values and full compute charges.
use super::super::{entry_edit::EntryFact, facts::RegionFault};
use super::*;
use crate::checkpoint_recovery::parallel_history::reuse_cases::ReuseFacts;
use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionStop, WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};
mod ceiling_pair;
mod code_identity;
mod computation;
mod declaration;
mod engagement;
mod execution;
mod seeded_world;
mod worker_matrix;
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
    worth_query_host::facade::primary_graph::advancement_observations_on_this_thread_for_test();
    only_run(demand(&request, &application))
}
fn case(change: Change) {
    case_at(change, 1, 1, true);
}
fn case_at(change: Change, prior: usize, current: usize, reuse: bool) {
    struct ReuseRestore(bool);
    impl Drop for ReuseRestore {
        fn drop(&mut self) {
            execution::REUSE_ON.set(self.0);
        }
    }
    let _reuse = ReuseRestore(execution::REUSE_ON.replace(reuse));
    let _guard = checkpoint_recovery_test_guard();
    use worth_query_host::facade::primary_graph::{
        advancement_observations_on_this_thread_for_test as observations,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    let previous = place(Placement::Leased(
        std::num::NonZeroUsize::new(prior).unwrap(),
    ));
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _restore = Restore(previous);
    let ceiling_family = matches!(change, Change::CarriedCeiling);
    execution::CARRIED_CEILING.set(ceiling_family);
    execution::COMPUTE_CEILING.set(false);
    struct CeilingRestore;
    impl Drop for CeilingRestore {
        fn drop(&mut self) {
            execution::CARRIED_CEILING.set(false);
            execution::COMPUTE_CEILING.set(false);
        }
    }
    let _ceiling = CeilingRestore;
    let mut model = facts();
    engagement::initialize(change, &mut model);
    if ceiling_family {
        let first = model
            .members()
            .min_by_key(|item| super::tree_work::identity(item.key))
            .unwrap()
            .key;
        model
            .items
            .iter_mut()
            .find(|item| item.key == first)
            .unwrap()
            .work = 20;
    }
    let code = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let reuse_control = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(reuse));
    let application = execution::install_with_controls(
        |graph| seed(&model, graph),
        code.clone(),
        reuse_control.clone(),
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before_value = model.expected_value();
    let before_work = declaration::computed_work(&model);
    let before = model.clone();
    observations();
    let (first, first_work) = only_run(demand(&request, &application));
    let prior_records = observations();
    assert!(
        prior_records.iter().any(|r| r.workers == Some(prior)),
        "the prior request dispatches at its named width"
    );
    assert!(
        prior_records.iter().all(|r| r.workers == Some(prior)),
        "every prior request dispatch has its named width"
    );
    assert_eq!(
        first.runs,
        [Run::Full(if reuse {
            Cause::FirstRun
        } else {
            Cause::Unretained
        })],
        "the first demand has no prior when enabled, and policy declines prior reuse when disabled"
    );
    assert_eq!(first.outcome.as_ref().unwrap().0, before_value.unwrap());
    assert_eq!(
        first_work,
        Some(before_work),
        "declared full compute and tree work"
    );
    let pair = (ceiling_family || matches!(change, Change::Gather))
        .then(|| ceiling_pair::branches(&application));
    let code_branch =
        matches!(change, Change::CodeIdentity).then(|| ceiling_pair::branches(&application)[0]);
    let request = match pair {
        Some([a, _]) => request.on_branch(a),
        None => match code_branch {
            Some(branch) => request.on_branch(branch),
            None => request,
        },
    };
    if let Some(edits) = change.apply(&mut model) {
        if let Some([_, b]) = pair {
            edit(
                &application.request(&principal, &scope).on_branch(b),
                &application,
                edits.clone(),
                2,
            );
        }
        edit(&request, &application, edits, 1);
    } else if !matches!(change, Change::CodeIdentity) {
        adjust(&request, &application, ODD_Y, 1);
    }
    let declared_work = declaration::computed_work(&model);
    if matches!(change, Change::EmptyStable | Change::CodeIdentity) {
        assert_eq!(
            model.expected_value(),
            before_value,
            "empty input replacement and code identity alone preserve the declared value"
        );
    } else {
        assert_ne!(
            model.expected_value(),
            before_value,
            "each varied input changes the modeled outcome"
        );
    }
    code.store(matches!(change, Change::CodeIdentity), Ordering::Relaxed);
    if matches!(
        change,
        Change::Join | Change::Leave | Change::DigestSwap | Change::Replacement
    ) {
        assert_ne!(
            declared_work, before_work,
            "membership and item-digest edits change modeled compute work"
        );
    }
    place(Placement::Leased(
        std::num::NonZeroUsize::new(current).unwrap(),
    ));
    observations();
    execution::COMPUTE_CEILING.set(ceiling_family);
    let (retained, retained_work) = only_run(demand(&request, &application));
    let records = observations();
    engagement::assert_kernels(change, &before, &model, reuse, &retained);
    engagement::assert_empty_history(change, &before, &model, &first, &retained);
    assert!(
        records.iter().any(|r| r.workers == Some(current)),
        "the reuse request reaches its declared current dispatch width"
    );
    assert!(
        records.iter().all(|r| r.workers == Some(current)),
        "every reused demand dispatch has the named width"
    );
    let retained_total = records.iter().fold((0, 0), |sum, r| {
        (
            sum.0 + r.report.charged_work(),
            sum.1 + r.report.charged_span(),
        )
    });
    if let Some([_, b]) = pair {
        reuse_control.store(false, Ordering::Relaxed);
        let (reference, _) = only_run(demand(
            &application.request(&principal, &scope).on_branch(b),
            &application,
        ));
        let full_total = observations().iter().fold((0, 0), |sum, r| {
            (
                sum.0 + r.report.charged_work(),
                sum.1 + r.report.charged_span(),
            )
        });
        assert_eq!(
            retained_total, full_total,
            "the same edited request has identical advancement work and span with reuse on or off"
        );
        assert_eq!(
            retained.outcome, reference.outcome,
            "reuse preserves the same request's result or typed stop"
        );
        println!("SAME_REQUEST_BILL:reuse={reuse}:edited_retained_work={}:edited_retained_span={}:edited_reuse_off_work={}:edited_reuse_off_span={}", retained_total.0, retained_total.1, full_total.0, full_total.1);
        if ceiling_family {
            ceiling_pair::assert_stop(&retained, &reference, &model, reuse);
            return;
        }
    }
    let (reference, fresh_work) = fresh(&model);
    let fresh_total = observations().iter().fold((0, 0), |sum, r| {
        (
            sum.0 + r.report.charged_work(),
            sum.1 + r.report.charged_span(),
        )
    });
    // A freshly seeded world has no prior edit/publication history. Its whole
    // advancement bill is a different input; only its declared computation
    // charge is compared below. Gather's same-history counterpart above pins
    // the reuse law for the entire advancement, including native owner reads.
    println!(
        "HISTORY_BILLS:edited_work={}:edited_span={}:new_installation_work={}:new_installation_span={}:declared={declared_work}",
        retained_total.0, retained_total.1, fresh_total.0, fresh_total.1
    );
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
    if model.expected_value().is_some() {
        assert_eq!(
            retained.outcome.as_ref().unwrap().1,
            reference.outcome.as_ref().unwrap().1,
            "reuse-on and reuse-off charge the same declared amount including preparation"
        );
        println!(
            "REUSE_LOGICAL_REMAINDER:{}",
            retained.outcome.as_ref().unwrap().1 - declared_work
        );
    }
    if model.expected_value().is_none() {
        let item = model.members().find(|item| item.fault).unwrap();
        let expected = super::tree_work::identity(item.key);
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
        let expected = if matches!(change, Change::Input | Change::EmptyStable) {
            Run::Full(Cause::InputChanged)
        } else if matches!(change, Change::CodeIdentity) {
            Run::Full(Cause::OtherInstallation)
        } else {
            Run::Incremental
        };
        if reuse {
            assert_eq!(retained.runs, [expected], "sensitive run kind");
        } else {
            assert_eq!(
                retained.runs,
                [Run::Full(Cause::Unretained)],
                "reuse-off does not retain a prior"
            );
        }
    }
}
#[test]
fn missing_reuse_inputs_are_executed_and_observed() {
    for change in [
        Change::EmptyBefore,
        Change::EmptyAfter,
        Change::EmptyStable,
        Change::CodeIdentity,
    ] {
        case(change);
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

#[test]
fn ceiling_inside_carried_work() {
    case(Change::CarriedCeiling);
}
