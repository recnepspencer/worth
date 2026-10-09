//! Immediate undo and redo preserve the admitted input and exact edit law.
use super::*;

#[test]
fn redo_on_the_undo_target_reapplies_the_first_application_byte_for_byte() {
    let _guard = checkpoint_recovery_test_guard();
    let first = applied::<true, 0>(false);
    let redo = applied::<true, 0>(true);
    let fresh = applied::<false, 3>(true);
    assert_eq!(
        first.outcome, redo.outcome,
        "redo lands byte-equal to the first application"
    );
    assert_eq!(
        redo.outcome, fresh.outcome,
        "redo: bits, typed outcome, charge and boundary"
    );
}

fn applied<const REUSE: bool, const MODE: u8>(redo: bool) -> OracleRun {
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install_variant::<REUSE, TOTALS_WORK, 1, MODE>(
        None,
        Default::default(),
        |graph| {
            case.before.seed(graph);
            super::super::super::super::entry_correction::seed_correction_grant(graph, SCOPE);
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
    let key = 0x612_5200_u64;
    let identities = preparation::Identities::<false>::encode(&key, &input).unwrap();
    let (program, _) = preparation::prepare(&application, &principal, &scope, &identities, None);
    let receipt = preparation::committed(application.compare_and_commit_program_action(
        program,
        &identities,
        std::convert::identity,
        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
    if redo {
        let handle = application
            .runtime()
            .mint_recovery_handle(&receipt)
            .unwrap();
        let inverse_key = key + 1;
        let inverse = preparation::Identities::<false>::encode(&inverse_key, &input).unwrap();
        let (program, authority) =
            preparation::prepare(&application, &principal, &scope, &inverse, Some(&handle));
        let handoff = aftermath::progress_admitted_undo(
            application
                .runtime()
                .admit_undo(handle, &authority.unwrap())
                .unwrap(),
        )
        .unwrap();
        let undo_receipt = preparation::committed(application.compare_and_commit_program_undo(
            program,
            &inverse,
            std::convert::identity,
            &handoff,
        ));
        let recovery =
            aftermath::WorthQueryRedoRecovery::from_completed_undo(handoff, &undo_receipt).unwrap();
        let redo_key = key + 2;
        let redo_identities = preparation::Identities::<false>::encode(&redo_key, &input).unwrap();
        let (program, authority) = preparation::prepare(
            &application,
            &principal,
            &scope,
            &redo_identities,
            Some(recovery.handle()),
        );
        let selected = application
            .runtime()
            .on_branch(application.current_world())
            .select()
            .unwrap();
        let intent = selected.derive_redo_intent(recovery.proved()).unwrap();
        let admitted = selected
            .admit_redo(recovery, &authority.unwrap(), &intent)
            .unwrap();
        drop(selected);
        let handoff = aftermath::progress_admitted_redo(admitted).unwrap();
        let redo_receipt = preparation::committed(application.compare_and_commit_program_redo(
            program,
            &redo_identities,
            std::convert::identity,
            &handoff,
        ));
        assert!(
            redo_receipt.aftermath_causality().is_some(),
            "redo enters the governed causality lane"
        );
        aftermath::consume_redo_progression(handoff).unwrap();
    }
    let (contacts, mut runs) = demand(&request, &application);
    assert_eq!((contacts, runs.len()), (1, 1));
    let run = runs.remove(0);
    if REUSE {
        assert_eq!(run.calls, case.after.expected_calls(Some(&case.before)));
        assert_eq!(run.runs, [Run::Incremental]);
        assert_eq!(
            run.tree_runs
                .iter()
                .map(|r| r.metrics().recombined_nodes)
                .sum::<u128>(),
            super::super::tree::expected_nodes(&case.after, Some(&case.before))
        );
    } else {
        assert_eq!(run.calls, case.after.expected_calls(None));
        assert_eq!(run.runs, [Run::Full(Cause::Unretained)]);
    }
    assert_eq!(
        demand(&request, &application).0,
        0,
        "the next unchanged step has no contact"
    );
    run
}
