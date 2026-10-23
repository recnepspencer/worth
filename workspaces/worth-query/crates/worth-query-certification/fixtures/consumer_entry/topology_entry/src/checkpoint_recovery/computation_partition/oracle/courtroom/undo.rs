//! Accepted correction is judged against the retained prior, through the public entry.
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::EntryCorrection;
mod after_publication;
use super::super::source_delivery::correction as preparation;

#[test]
fn recorded_correction_reuses_the_retained_partitions() {
    let _guard = checkpoint_recovery_test_guard();
    let reused = directly_undone::<true, 0>();
    let fresh = directly_undone::<false, 3>();
    assert_eq!(
        reused.outcome, fresh.outcome,
        "correction: result bits, typed outcome and charged work"
    );
}
fn directly_undone<const REUSE: bool, const MODE: u8>() -> OracleRun {
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install_variant::<REUSE, TOTALS_WORK, 1, MODE>(
        None,
        Default::default(),
        |graph| {
            case.before.seed(graph);
            crate::checkpoint_recovery::computation_partition::oracle::courtroom::correction_grant::seed(graph, SCOPE);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    demand(&request, &application);
    let input = EntryCorrection {
        scope_key: SCOPE.to_owned(),
        entry: 1,
        mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
        expected: None,
    };
    let key = 0x612_5000_u64;
    let receipt = preparation::source(&application, &request, input, key);
    let undo_receipt = preparation::reverse(&application, &request, &receipt, "101863425");
    assert!(
        undo_receipt.aftermath_causality().is_some(),
        "this is a real accepted correction commit"
    );
    let (contacts, mut runs) = demand(&request, &application);
    assert_eq!(
        contacts, 1,
        "the managed producer judges the undo's touched facts once"
    );
    assert_eq!(runs.len(), 1);
    let run = runs.remove(0);
    if REUSE {
        assert_eq!(
            run.calls,
            case.before.expected_calls(Some(&case.before)),
            "the undo restores the complete retained gathered basis"
        );
        assert_eq!(run.calls.kernels, 0);
        assert_eq!(tree_work::completed_nodes(&run), 0);
        assert_eq!(run.runs, [Run::Incremental]);
    } else {
        assert_eq!(run.runs, [Run::Full(Cause::Unretained)]);
    }
    let (contacts, next) = demand(&request, &application);
    assert_eq!(
        (contacts, next.len()),
        (0, 0),
        "the step after undo is unchanged"
    );
    run
}
