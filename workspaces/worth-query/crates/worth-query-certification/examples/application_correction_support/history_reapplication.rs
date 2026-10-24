//! Canonical alternatives survive native projection disposal without reviving expired history.
use super::{authentication::*, edit::Value, host};
use std::num::NonZeroUsize;
use worth_query_host::facade::{application_entry::*, primary_graph::*};

fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN)
}

pub fn run() {
    consumed_edge_survives_reconstruction();
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "history-source");
    let reversal = reverse(&host, &source);
    let reversed_commit = reversal.commit_reference().clone();
    let reversed_world = reversal
        .committed_product_publication()
        .composite_commit()
        .clone();
    let root = reversal
        .aftermath_causality()
        .unwrap()
        .recorded_source()
        .coordinates();
    let baseline = entries(&host, &source);
    assert_eq!(baseline.len(), 1);
    assert_eq!(baseline[0].performed_commit(), &reversed_commit);
    assert_eq!(baseline[0].source(), &root);

    discard(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request.corrections().select(&source, limits()).unwrap();
    assert!(
        matches!(
            request
                .corrections()
                .alternatives(&selected, limits(), NonZeroUsize::MIN),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::History(
                    WorthQueryAftermathCausalityReadDenial::IndexUnavailable { .. }
                )
            ))
        ),
        "ordinary lookup must expose the missing native generation"
    );
    drop(selected);
    rebuild(&host);
    assert_eq!(
        entries(&host, &source),
        baseline,
        "explicit reconstruction derives the same rows from canonical facts"
    );

    drop(reversal);
    host::edit(&host, Value::Secondary(19), "history-expiry-first");
    host::edit(&host, Value::Secondary(23), "history-expiry-second");
    let history = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .history(&phase, host.runtime().current_world(), None)
        })
        .unwrap()
        .unwrap();
    assert!(
        history
            .entries()
            .all(|entry| entry.selected_commit() != &reversed_world),
        "real native history retirement must precede the unavailable assertion"
    );
    drop(history);
    assert_eq!(
        entries(&host, &source),
        baseline,
        "permitted causal metadata remains after optional performed history expires"
    );
    assert_expired_alternative(&host, &source, &baseline[0]);
    discard(&host);
    rebuild(&host);
    assert_eq!(entries(&host, &source), baseline);
    assert_expired_alternative(&host, &source, &baseline[0]);

    drop(source);
    host::edit(&host, Value::Secondary(29), "history-source-expiry-first");
    host::edit(&host, Value::Secondary(31), "history-source-expiry-second");
    assert_expired_source(&host, &root);
    discard(&host);
    rebuild(&host);
    assert_expired_source(&host, &root);
    println!("canonical correction indexes reconstruct exactly; native expiry remains unavailable");
}

fn consumed_edge_survives_reconstruction() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "history-consumption-source");
    let reversal = reverse(&host, &source);
    let alternative = super::tree_reapplication::alternative(&host, &source, &reversal, 1);
    let reapplied =
        super::tree_reapplication::reapply(&host, &alternative, "history-consumption-reapply");
    assert_eq!(
        reapplied
            .aftermath_causality()
            .unwrap()
            .correction_lineage()
            .active_reversal()
            .unwrap()
            .relational(),
        reversal.commit_reference()
    );
    assert_eq!(
        host::read(&host),
        [super::query::Row {
            primary: 11,
            secondary: 7
        }]
    );
    drop(alternative);
    let baseline = entries(&host, &source);
    assert_eq!(baseline.len(), 1);
    assert_eq!(baseline[0].performed_commit(), reversal.commit_reference());
    discard(&host);
    rebuild(&host);
    assert_eq!(
        entries(&host, &source),
        baseline,
        "reconstructing a consumed reversal preserves its canonical alternative"
    );
    let alternative = super::tree_reapplication::alternative(&host, &source, &reversal, 1);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    assert!(
        matches!(
            request
                .corrections()
                .reapply(&alternative)
                .idempotency("history-consumption-distinct-key")
                .limits(limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::NoActiveReversal
            ))
        ),
        "rebuilding consumption access cannot reopen a consumed reversal"
    );
    assert_eq!(
        host::read(&host),
        [super::query::Row {
            primary: 11,
            secondary: 7
        }]
    );
}

fn entries(
    host: &host::Host,
    source: &WorthQueryApplicationCommitReceipt,
) -> Vec<WorthQueryCorrectionAlternativeRef> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request.corrections().select(source, limits()).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits(), NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    page.entries().to_vec()
}

fn assert_expired_alternative(
    host: &host::Host,
    source: &WorthQueryApplicationCommitReceipt,
    alternative: &WorthQueryCorrectionAlternativeRef,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request.corrections().select(source, limits()).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits(), NonZeroUsize::MIN)
        .unwrap();
    let denial = page
        .select(alternative, limits())
        .err()
        .expect("expired selection must refuse");
    let WorthQueryApplicationCorrectionDenial::Derivation(
        WorthQueryCorrectionDerivationDenial::Ancestry(denial),
    ) = denial
    else {
        panic!("unexpected expired alternative denial: {denial:?}")
    };
    assert!(
        matches!(
            denial,
            RuntimeWorldServiceDenial::Denied(ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::SourceUnavailable
            ))
        ),
        "metadata cannot supply expired native selection authority: {denial:?}"
    );
}

fn assert_expired_source(host: &host::Host, source: &WorthQueryCorrectionSourceCoordinates) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let denial = request
        .corrections()
        .select_coordinates(source, limits())
        .err()
        .expect("expired source must refuse");
    let WorthQueryApplicationCorrectionDenial::Source(
        WorthQueryCorrectionSelectionDenial::Ancestry(denial),
    ) = denial
    else {
        panic!("unexpected expired source denial: {denial:?}")
    };
    assert!(
        matches!(
            denial,
            RuntimeWorldServiceDenial::Denied(ProductBranchAncestorSelectionDenial::History(
                CompositeAncestorSelectionDenial::SourceUnavailable
            ))
        ),
        "derived reconstruction cannot restore expired original history: {denial:?}"
    );
}

fn discard(host: &host::Host) {
    let removed = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .release_correction_history_indexes_for_test(&phase)
        })
        .unwrap()
        .unwrap();
    assert!(
        removed > 0,
        "the control must dispose actual native generations"
    );
}
fn rebuild(host: &host::Host) {
    let rebuilt = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .rebuild_correction_history_indexes_for_test(&phase, host.runtime().current_world())
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        rebuilt.generations.len(),
        3,
        "all correction selectors are reconstructed"
    );
}
fn reverse(
    host: &host::Host,
    source: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request.corrections().select(source, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("history-reverse")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh reversal prepares")
    };
    match prepared.publish().unwrap() {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("fresh reversal commits: {other:?}"),
    }
}
