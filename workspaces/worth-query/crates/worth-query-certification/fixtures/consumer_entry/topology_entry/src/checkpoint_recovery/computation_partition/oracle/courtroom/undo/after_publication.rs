//! Undo may seal only a commit whose direct parent is its target.
use super::*;

#[test]
fn undo_after_an_output_publication_refuses_without_losing_the_retained_prior() {
    let _guard = checkpoint_recovery_test_guard();
    let case = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap();
    let application = installation::install_variant::<true, TOTALS_WORK, 1, 0>(
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
    let key = 0x612_5100_u64;
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
    let (contacts, runs) = demand(&request, &application);
    assert_eq!((contacts, runs.len()), (1, 1));
    assert_eq!(runs[0].calls, case.after.expected_calls(Some(&case.before)));
    let selected_before = application
        .runtime()
        .on_branch(application.current_world())
        .select()
        .unwrap()
        .product()
        .selected_commit()
        .clone();
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
    let owner_before = application.producer_contacts_on_this_thread_for_test();
    let outcome = application.compare_and_commit_program_undo(
        program,
        &inverse,
        std::convert::identity,
        &handoff,
    );
    assert!(
        matches!(&outcome, primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial)
        if denial.kind() == primary_graph::WorthQueryApplicationCommitDenialKind::ProviderRejected
            && denial.stage() == primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution
            && denial.detail() == Some("the admitted aftermath parent is no longer the current Relational head")),
        "an intervening publication is refused before any effect: {outcome:?}"
    );
    assert_eq!(
        application.producer_contacts_on_this_thread_for_test(),
        owner_before,
        "refused undo/redo made zero owner calls"
    );
    let selected_after = application
        .runtime()
        .on_branch(application.current_world())
        .select()
        .unwrap()
        .product()
        .selected_commit()
        .clone();
    assert_eq!(selected_before, selected_after, "refusal publishes nothing");
    assert_eq!(
        demand(&request, &application).0,
        0,
        "refusal leaves the current prior eligible"
    );
    let next_key = key + 2;
    let next = preparation::Identities::<false>::encode(&next_key, &input).unwrap();
    let (program, _) = preparation::prepare(&application, &principal, &scope, &next, None);
    preparation::committed(application.compare_and_commit_program_action(
        program,
        &next,
        std::convert::identity,
        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
    let (contacts, runs) = demand(&request, &application);
    assert_eq!((contacts, runs.len()), (1, 1));
    assert_eq!(runs[0].calls, case.before.expected_calls(Some(&case.after)));
    assert_eq!(runs[0].runs, [Run::Incremental]);
}
