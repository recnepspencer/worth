//! Output publication preserves correction eligibility; changed source truth does not.
use super::*;
use worth_query_host::facade::application_entry::*;

#[test]
fn recorded_correction_after_output_publication_preserves_the_retained_prior() {
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
            super::super::correction_grant::seed(graph, SCOPE);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    demand(&request, &application);
    let source = preparation::source(
        &application,
        &request,
        EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 1,
            mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
            expected: None,
        },
        0x612_5100,
    );
    let (contacts, runs) = demand(&request, &application);
    assert_eq!((contacts, runs.len()), (1, 1));
    assert_eq!(runs[0].calls, case.after.expected_calls(Some(&case.before)));
    let published_output = crate::checkpoint_recovery::support::host_call(&application, |phase| {
        application
            .runtime()
            .on_branch(application.current_world())
            .select(phase)
    })
    .unwrap();
    assert_ne!(
        published_output.product().selected_commit(),
        source.committed_product_publication().composite_commit(),
        "the producer actually published an intervening World occurrence"
    );
    let inverse = preparation::reverse(&application, &request, &source, "101863681");
    let fact = inverse.aftermath_causality().unwrap();
    assert_ne!(fact.parent(), source.commit_reference());
    assert_eq!(
        fact.recorded_source().relational(),
        source.commit_reference()
    );
    let (contacts, runs) = demand(&request, &application);
    assert_eq!((contacts, runs.len()), (1, 1));
    assert_eq!(runs[0].calls, case.before.expected_calls(Some(&case.after)));
    assert_eq!(runs[0].runs, [Run::Incremental]);
    assert_eq!(demand(&request, &application).0, 0);
}

#[test]
fn recorded_correction_refuses_intervening_source_value_changes() {
    let _guard = checkpoint_recovery_test_guard();
    let model = Model::law_cases()
        .into_iter()
        .find(|case| case.name == "Value change")
        .unwrap()
        .before;
    let application = installation::install(|graph| {
        model.seed(graph);
        super::super::correction_grant::seed(graph, SCOPE);
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let source = preparation::source(
        &application,
        &request,
        EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 1,
            mask: 2.0_f64.to_bits() ^ 3.0_f64.to_bits(),
            expected: None,
        },
        0x612_5200,
    );
    let selected = request
        .corrections()
        .select(&source, preparation::limits())
        .unwrap();
    let changed = preparation::source(
        &application,
        &request,
        EntryCorrection {
            scope_key: SCOPE.to_owned(),
            entry: 1,
            mask: 3.0_f64.to_bits() ^ 4.0_f64.to_bits(),
            expected: None,
        },
        0x612_5201,
    );
    let owner_before = application.producer_contacts_on_this_thread_for_test();
    let refusal = request
        .corrections()
        .reverse(&selected)
        .idempotency("101863938")
        .limits(preparation::limits())
        .prepare()
        .err()
        .expect("the retained mask must not mutate a changed source value");
    assert!(matches!(
        refusal,
        WorthQueryApplicationCorrectionDenial::Derivation(
            primary_graph::WorthQueryCorrectionDerivationDenial::SourceDiverged
        )
    ));
    assert_eq!(
        application.producer_contacts_on_this_thread_for_test(),
        owner_before
    );
    let current = crate::checkpoint_recovery::support::host_call(&application, |phase| {
        application
            .runtime()
            .on_branch(application.current_world())
            .select(phase)
    })
    .unwrap();
    assert_eq!(
        current.product().selected_commit(),
        changed.committed_product_publication().composite_commit(),
        "refused correction publishes no occurrence"
    );
}
