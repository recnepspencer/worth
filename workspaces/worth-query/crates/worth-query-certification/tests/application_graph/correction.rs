//! Exact performed selection uses real public installation, publication and disclosure.
// The example owns this model and lints all of it; this court uses a part.
#[allow(dead_code, unused_imports)]
#[path = "../../examples/application_correction_support/mod.rs"]
mod model;
use model::{authentication::*, edit::Value, host};
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::WorthQueryCorrectionDerivationDenial;

#[test]
fn retained_reapplication_uses_explicit_alternatives_and_active_consumption() {
    model::tree_reapplication::run();
}

#[test]
fn reapplication_preserves_branch_authority_and_current_source_decisions() {
    model::tree_controls::run();
}

#[test]
fn reapplication_completes_original_required_outputs_and_retries() {
    model::required_reapplication::run();
}

#[test]
fn correction_history_rebuild_preserves_causality_and_source_expiry() {
    model::history_reapplication::run();
}

/// A hundred adoptions outgrow the example's whole-run stall watch; the
/// example still runs this court alone as `adoption-cycles`.
#[test]
fn fork_head_adoption_retains_linear_history_and_corrections() {
    model::fork_head_adoption::run();
}

#[test]
fn performed_source_survives_operation_close_and_receipt_copy() {
    let host = host::install();
    let receipt = host::edit(&host, Value::Primary(11), "original");
    let copy = receipt.clone();
    drop(receipt);
    host::edit(&host, Value::Secondary(19), "disjoint");
    assert_eq!(
        host::read(&host),
        [model::query::Row {
            primary: 11,
            secondary: 19
        }]
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(
            &copy,
            WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN),
        )
        .expect("native ancestry and fresh disclosure select the performed source after close");
    assert_eq!(
        request
            .corrections()
            .reverse(&selected)
            .inspect(WorthQueryCorrectionHistoryLimits::new(
                None,
                NonZeroUsize::MIN
            ))
            .unwrap(),
        WorthQueryCorrectionApplicability::ReadyForPreparation
    );
}

#[test]
fn changed_independent_handler_dependency_refuses_an_unchanged_afterimage() {
    let host = host::install();
    let receipt = host::edit(&host, Value::Primary(11), "original");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    request
        .corrections()
        .reverse(&selected)
        .inspect(limits)
        .unwrap();
    host::edit(&host, Value::Ceiling(50), "dependency");
    assert_eq!(
        host::read(&host),
        [model::query::Row {
            primary: 11,
            secondary: 7
        }]
    );
    assert!(matches!(
        request.corrections().reverse(&selected).inspect(limits),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceDiverged
        ))
    ));
}

#[test]
fn filtered_current_subject_refuses_history_even_with_a_genuine_receipt() {
    let host = host::install();
    let receipt = host::edit(&host, Value::Primary(11), "original");
    host::edit(&host, Value::ReadEnabled(false), "hide");
    assert!(host::read(&host).is_empty());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let selected = host
        .runtime()
        .request(&principal, &scope)
        .corrections()
        .select(
            &receipt,
            WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN),
        );
    assert!(
        selected.is_err(),
        "receipt possession cannot disclose a filtered subject"
    );
}
