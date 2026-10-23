//! Real source owners, explicit historical alternatives, and branch-local consumption.
use super::{
    authentication::*, correction::RecordedEdit, edit::Value, host, program::*, query::Row,
    schema::NumericSchema,
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{application_entry::*, primary_graph::*};

pub(super) fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN)
}

pub fn run() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "tree-source");
    let first_reverse = reverse(&host, &source, "tree-reverse-source");
    host::edit(&host, Value::Secondary(19), "tree-disjoint-before-reapply");
    let first_alternative = alternative(&host, &source, &first_reverse, 1);
    let first_reapply = reapply(&host, &first_alternative, "tree-reapply-first");
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 19
        }]
    );
    assert_reapplication(
        &first_reapply,
        &source,
        &source,
        &first_reverse,
        &first_reverse,
    );

    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let old_source = request.corrections().select(&source, limits()).unwrap();
    assert!(
        matches!(
            request
                .corrections()
                .reverse(&old_source)
                .idempotency("repeat-old-source-reversal")
                .limits(limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::SourceAlreadyReversed
            ))
        ),
        "the next reversal must name the new performed reapplication, not old S"
    );

    let second_reverse = reverse(&host, &first_reapply, "tree-reverse-reapplication");
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    let parent = host::edit(
        &host,
        Value::Secondary(23),
        "tree-disjoint-after-second-reverse",
    );
    let old_alternative = alternative(&host, &source, &first_reverse, 2);
    let before_retry = super::required_output_courts::head(&host);
    let retry = reapply(&host, &old_alternative, "tree-reapply-first");
    assert_eq!(
        super::required_output_courts::head(&host),
        before_retry,
        "exact replay must not publish another World occurrence"
    );
    assert_eq!(
        retry.commit_reference(),
        first_reapply.commit_reference(),
        "same-key replay resolves its original selected and active edges"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 23
        }],
        "retry must not consume the later active reversal"
    );

    let recent_alternative = alternative(&host, &source, &second_reverse, 2);
    let competing = request
        .corrections()
        .reapply(&recent_alternative)
        .idempotency("tree-competing-alternative")
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(competing) = competing else {
        panic!("both alternatives prepare while V is active")
    };
    let second_reapply = reapply(&host, &old_alternative, "tree-reapply-older-alternative");
    let losing = competing.publish().unwrap();
    assert!(
        matches!(&losing, WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(denial))
        if denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale),
        "only one prepared choice can consume V: {losing:?}"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 23
        }]
    );
    assert_eq!(
        second_reapply.commit_reference().parents,
        vec![parent.commit_reference().commit_id]
    );
    assert_reapplication(
        &second_reapply,
        &source,
        &source,
        &first_reverse,
        &second_reverse,
    );
    assert_reapplication(
        &first_reapply,
        &source,
        &source,
        &first_reverse,
        &first_reverse,
    );

    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let consumed = alternative(&host, &source, &second_reverse, 2);
    assert!(
        matches!(
            request
                .corrections()
                .reapply(&consumed)
                .idempotency("tree-distinct-key")
                .limits(limits())
                .prepare(),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::NoActiveReversal
            ))
        ),
        "a different alternative and key cannot consume the same active reversal twice"
    );
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 23
        }]
    );
    println!("retained source reapplication, complete alternative pages, exact replay and active-edge consumption passed");
}

pub(super) fn reverse(
    host: &host::Host,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request.corrections().select(source, limits()).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh reversal must prepare")
    };
    let outcome = prepared.publish().unwrap();
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("fresh reversal must commit: {other:?}"),
    }
}

pub(super) fn alternative<'a>(
    host: &'a host::Host,
    source: &WorthQueryApplicationCommitReceipt,
    reversal: &WorthQueryApplicationCommitReceipt,
    expected_count: usize,
) -> WorthQuerySelectedCorrectionAlternative<'a, NumericSchema> {
    alternative_on(
        host,
        host.runtime().current_world(),
        source,
        reversal,
        expected_count,
    )
}

pub(super) fn alternative_on<'a>(
    host: &'a host::Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    source: &WorthQueryApplicationCommitReceipt,
    reversal: &WorthQueryApplicationCommitReceipt,
    expected_count: usize,
) -> WorthQuerySelectedCorrectionAlternative<'a, NumericSchema> {
    let (page, chosen) = {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope).on_branch(branch);
        let selected = request.corrections().select(source, limits()).unwrap();
        let mut page = request
            .corrections()
            .alternatives(&selected, limits(), NonZeroUsize::MIN)
            .unwrap();
        let mut entries = page.entries().to_vec();
        assert!(entries.len() <= 1, "the requested page width is one");
        if expected_count == 2 {
            assert!(
                page.continuation().is_some(),
                "two alternatives require continuation at page width one"
            );
        }
        if page.continuation().is_some() {
            assert!(matches!(
                page.select(&entries[0], limits()),
                Err(WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::AlternativeResolutionIncomplete
                ))
            ));
        }
        while let Some(continuation) = page.continuation() {
            let next = request
                .corrections()
                .continue_alternatives(&selected, limits(), NonZeroUsize::MIN, continuation)
                .unwrap();
            assert!(
                next.entries().len() <= 1,
                "every continued page respects its requested width"
            );
            entries.extend_from_slice(next.entries());
            page = next;
        }
        assert_eq!(
            entries.len(),
            expected_count,
            "ordinary edits preserve every canonical alternative"
        );
        for (position, entry) in entries.iter().enumerate() {
            assert!(
                !entries[..position].contains(entry),
                "continuation cannot repeat a row"
            );
        }
        let chosen = entries
            .iter()
            .find(|entry| entry.performed_commit() == reversal.commit_reference())
            .expect("the exact historical reversal is listed");
        let coordinates = chosen.source().clone();
        (page.into_page(), coordinates)
    };
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    host.runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(branch)
        .corrections()
        .select_alternative_coordinates(&page, &chosen, limits())
        .expect("detached history resolves the chosen source under a fresh request")
}

pub(super) fn reapply(
    host: &host::Host,
    alternative: &WorthQuerySelectedCorrectionAlternative<'_, NumericSchema>,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    reapply_on(host, host.runtime().current_world(), alternative, key)
}

pub(super) fn reapply_on(
    host: &host::Host,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
    alternative: &WorthQuerySelectedCorrectionAlternative<'_, NumericSchema>,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope).on_branch(branch);
    let prepared = request
        .corrections()
        .reapply(alternative)
        .idempotency(key)
        .limits(limits())
        .prepare_operation::<RecordedEdit, Program, Action>(host)
        .unwrap();
    let advance = match prepared {
        WorthQueryOperationCorrectionPreparation::Prepared(prepared) => prepared.publish().unwrap(),
        WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
        _ => panic!("reapplication must preserve an ordinary source owner"),
    };
    let completed = match advance {
        WorthQuerySourceOnlyAdvance::Completed(completed) => completed,
        WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
            panic!("reapplication source refused: {denial:?}")
        }
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
            panic!("reapplication terminal: {:?}", terminal.native_outcome())
        }
        WorthQuerySourceOnlyAdvance::Progress(_) => {
            panic!("numeric source has no pending delivery")
        }
    };
    let (_, evidence) = completed
        .close_with_evidence(&request)
        .unwrap_or_else(|(_, denial)| panic!("reapplication normal close: {denial:?}"));
    match evidence {
        WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. } => receipt,
    }
}

fn assert_reapplication(
    receipt: &WorthQueryApplicationCommitReceipt,
    root: &WorthQueryApplicationCommitReceipt,
    source: &WorthQueryApplicationCommitReceipt,
    selected: &WorthQueryApplicationCommitReceipt,
    active: &WorthQueryApplicationCommitReceipt,
) {
    let fact = receipt.aftermath_causality().unwrap();
    assert_eq!(fact.role(), WorthQueryAftermathCausalRole::RecordedReapply);
    let lineage = fact.correction_lineage();
    assert_eq!(lineage.root().relational(), root.commit_reference());
    assert_eq!(
        lineage.root().world(),
        root.committed_product_publication().composite_commit()
    );
    assert_eq!(
        fact.recorded_source().relational(),
        source.commit_reference()
    );
    assert_eq!(
        lineage.selected_alternative().unwrap().relational(),
        selected.commit_reference()
    );
    assert_eq!(
        lineage.selected_alternative().unwrap().world(),
        selected.committed_product_publication().composite_commit()
    );
    assert_eq!(
        lineage.active_reversal().unwrap().relational(),
        active.commit_reference()
    );
    assert_eq!(
        lineage.active_reversal().unwrap().world(),
        active.committed_product_publication().composite_commit()
    );
}
