//! Governed undo is judged against the retained prior, through the public entry.
use super::*;
use crate::checkpoint_recovery::computation_partition::entry_correction::{
    CorrectEntryValue, EntryCorrection, EntryCorrectionBinding,
};
use worth_query_decl::facade::application_schema::{
    ApplicationReadableScalarValueBinding, U64ApplicationValueBinding,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::provisional_aftermath as aftermath;
mod after_publication;
mod owner_refusals;
mod preparation;
mod redo;

#[test]
fn undo_directly_on_its_target_reuses_the_retained_partitions() {
    let _guard = checkpoint_recovery_test_guard();
    let reused = directly_undone::<true, 0>();
    let fresh = directly_undone::<false, 3>();
    assert_eq!(
        reused.outcome, fresh.outcome,
        "undo: result bits, typed outcome and charged work"
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
            super::super::super::entry_correction::seed_correction_grant(graph, SCOPE);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    demand(&request, &application);
    let input = EntryCorrection {
        scope_key: SCOPE.to_owned(),
        entry: 1,
        mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
    };
    let key = 0x612_5000_u64;
    let identities = preparation::Identities::<false>::encode(&key, &input).unwrap();
    let (program, _) = preparation::prepare(&application, &principal, &scope, &identities, None);
    let receipt = preparation::committed(application.compare_and_commit_program_action(
        program,
        &identities,
        std::convert::identity,
        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
    let handle = application
        .runtime()
        .mint_recovery_handle(&receipt)
        .unwrap();
    let field = receipt
        .retained_preimage()
        .unwrap()
        .field_for(super::super::super::facts::EntryValueBits::reference::<
            CheckpointSchema,
        >())
        .unwrap();
    let original = U64ApplicationValueBinding::decode(field.value()).unwrap();
    assert_eq!(
        original,
        2.0_f64.to_bits(),
        "the inverse uses the receipt's exact prior truth"
    );
    let inverse = input.clone();
    let inverse_key = key + 1;
    let inverse_identities =
        preparation::Identities::<false>::encode(&inverse_key, &inverse).unwrap();
    let (program, authority) = preparation::prepare(
        &application,
        &principal,
        &scope,
        &inverse_identities,
        Some(&handle),
    );
    let undo = application
        .runtime()
        .admit_undo(handle, &authority.unwrap())
        .unwrap();
    let handoff = aftermath::progress_admitted_undo(undo).unwrap();
    let undo_receipt = preparation::committed(application.compare_and_commit_program_undo(
        program,
        &inverse_identities,
        std::convert::identity,
        &handoff,
    ));
    assert!(
        undo_receipt.aftermath_causality().is_some(),
        "this is a real governed undo commit"
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
        assert_eq!(
            run.tree_runs
                .iter()
                .map(|r| r.metrics().recombined_nodes)
                .sum::<u128>(),
            0
        );
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
