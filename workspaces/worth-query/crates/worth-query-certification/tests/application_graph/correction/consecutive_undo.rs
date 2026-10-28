//! Fresh public reversal of original performed sources, without head restoration.
use super::*;
use model::tree_reapplication::{alternative, reapply, reverse};
use worth_query_execution::facade::primary_graph::{
    fact_verification_on_this_thread_for_test, WorthQueryExecutionObservation,
};
use worth_query_host::facade::product::WorthQueryApplicationCommitReceipt;
#[path = "consecutive_undo/adjacency.rs"]
mod adjacency;

fn diverged(host: &host::Host, source: &WorthQueryApplicationCommitReceipt) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let actual = request.corrections().reverse(&selected).inspect(limits);
    let observed = fact_verification_on_this_thread_for_test();
    match actual {
        Err(cause) => assert!(
            matches!(
                &cause,
                WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::SourceDiverged
                )
            ),
            "actual refusal: {cause:?}; unsupported overlap: {:?}",
            observed.correction_restored_support_last_unsupported
        ),
        Ok(_) => panic!(
            "expected source divergence; unsupported overlap: {:?}",
            observed.correction_restored_support_last_unsupported
        ),
    }
}

fn pin_current(
    host: &host::Host,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let pin = host
        .runtime()
        .request(&principal, &scope)
        .retain_current_read()
        .unwrap();
    assert_eq!(
        pin.selected_commit(),
        receipt.committed_product_publication().composite_commit(),
        "retain the actual performed endpoint, not a reconstructed descriptor"
    );
    pin
}

#[test]
fn original_sources_reverse_consecutively_after_exact_later_reversals() {
    for three in [false, true] {
        let host = host::install();
        host::edit(&host, Value::Primary(3), "initial-three");
        let a = host::edit(&host, Value::Primary(7), "original-a");
        let _a_pin = pin_current(&host, &a);
        let b = host::edit(&host, Value::Primary(8), "original-b");
        let _b_pin = pin_current(&host, &b);
        let _third_pins = if three {
            let c = host::edit(&host, Value::Primary(9), "original-c");
            let c_pin = pin_current(&host, &c);
            let undone = reverse(&host, &c, "reverse-original-c");
            let reversal_pin = pin_current(&host, &undone);
            assert_eq!(
                undone
                    .aftermath_causality()
                    .unwrap()
                    .recorded_source()
                    .relational(),
                c.commit_reference()
            );
            assert_eq!(host::read(&host)[0].primary, 8);
            Some((c_pin, reversal_pin))
        } else {
            None
        };
        let undone = reverse(&host, &b, "reverse-original-b");
        let _reversal_pin = pin_current(&host, &undone);
        assert_eq!(
            undone
                .aftermath_causality()
                .unwrap()
                .recorded_source()
                .relational(),
            b.commit_reference()
        );
        assert_eq!(host::read(&host)[0].primary, 7);
        host::edit(&host, Value::Secondary(19), "unrelated-field");
        let undone = reverse(&host, &a, "reverse-original-a");
        assert_eq!(
            undone
                .aftermath_causality()
                .unwrap()
                .recorded_source()
                .relational(),
            a.commit_reference()
        );
        assert_eq!(
            host::read(&host),
            [model::query::Row {
                primary: 3,
                secondary: 19
            }]
        );
    }
}

#[test]
fn live_equal_and_reapplied_occurrences_block_until_that_exact_occurrence_reverses() {
    let host = host::install();
    host::edit(&host, Value::Primary(3), "equal-initial");
    let a = host::edit(&host, Value::Primary(7), "equal-a");
    let equal = host::edit(&host, Value::Primary(7), "equal-b");
    diverged(&host, &a);
    reverse(&host, &equal, "reverse-equal-b");
    reverse(&host, &a, "reverse-equal-a");
    assert_eq!(host::read(&host)[0].primary, 3);

    let a = host::edit(&host, Value::Primary(7), "reapply-a");
    let b = host::edit(&host, Value::Primary(8), "reapply-b");
    let rb = reverse(&host, &b, "reverse-reapply-b");
    let choice = alternative(&host, &b, &rb, 1);
    let reapplied = reapply(&host, &choice, "new-exact-b-occurrence");
    diverged(&host, &a);
    reverse(&host, &reapplied, "reverse-new-exact-b");
    assert_eq!(host::read(&host)[0].primary, 7);
    reverse(&host, &a, "reverse-after-new-exact-b");
    assert_eq!(host::read(&host)[0].primary, 3);
}

#[test]
fn restored_values_do_not_cancel_independent_dependency_writes_or_sibling_history() {
    let host = host::install();
    host::edit(&host, Value::Primary(3), "dependency-initial");
    let a = host::edit(&host, Value::Primary(7), "dependency-a");
    let _dependency_a_pin = pin_current(&host, &a);
    let b = host::edit(&host, Value::Primary(8), "dependency-b");
    let _dependency_b_pin = pin_current(&host, &b);
    let reversed_b = reverse(&host, &b, "dependency-reverse-b");
    let _reversed_b_pin = pin_current(&host, &reversed_b);
    host::edit(&host, Value::Ceiling(50), "independent-dependency");
    diverged(&host, &a);
    host::edit(&host, Value::Ceiling(100), "equal-restored-dependency");
    assert_eq!(host::read(&host)[0].primary, 7);
    diverged(&host, &a);

    let host = host::install();
    host::edit(&host, Value::Primary(3), "sibling-initial");
    let a = host::edit(&host, Value::Primary(7), "sibling-a");
    let _sibling_a_pin = pin_current(&host, &a);
    let b = host::edit(&host, Value::Primary(8), "sibling-b");
    let _sibling_b_pin = pin_current(&host, &b);
    let sibling = host
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(host.runtime().current_world())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(sibling);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(&b, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("sibling-reverse-b")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("genuine sibling reversal prepares");
    };
    assert!(matches!(
        prepared.publish().unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    diverged(&host, &a);
    assert_eq!(host::read(&host)[0].primary, 8);
    assert_eq!(host::read_on(&host, sibling)[0].primary, 7);
    reverse(&host, &b, "main-reverse-b");
    reverse(&host, &a, "main-reverse-a");
    assert_eq!(host::read(&host)[0].primary, 3);
}
