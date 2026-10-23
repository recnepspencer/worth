//! Authored source selection through the ordinary request and a real performed receipt.
mod application_correction_support;
use application_correction_support::publication_courts::*;
use application_correction_support::{authentication::*, edit::Value, host, query::Row};
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionPreparation,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryCorrectionApplicability, WorthQueryCorrectionHistoryLimits,
};
use worth_query_host::facade::primary_graph::WorthQueryCorrectionDerivationDenial;

fn main() {
    std::thread::Builder::new()
        .name("application-correction".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(run)
        .expect("start the application thread")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
}

fn run() {
    application_correction_support::adoption_courts::run();
    selected_source_after_disjoint_edit();
    independent_semantic_dependency();
    reverse_after_disjoint_and_replay();
    reverse_inherited_fork_and_refuse_key_target_drift();
    stale_prepared_target_refuses();
    fresh_actor_can_reverse_after_original_actor_revocation();
    foreign_runtime_receipt_refuses();
}

fn selected_source_after_disjoint_edit() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    host::edit(&host, Value::Secondary(19), "disjoint");
    assert_eq!(
        host::read(&host),
        [Row {
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
            &source.clone(),
            WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN),
        )
        .expect("a closed performed source remains selectable after a disjoint edit");
    assert_eq!(
        request
            .corrections()
            .reverse(&selected)
            .inspect(history_limits())
            .unwrap(),
        WorthQueryCorrectionApplicability::ReadyForPreparation
    );
    host::edit(&host, Value::Primary(13), "overlap");
    assert!(matches!(
        request
            .corrections()
            .reverse(&selected)
            .inspect(history_limits()),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceDiverged
        ))
    ));
    host::edit(&host, Value::ReadEnabled(false), "hide");
    assert!(host::read(&host).is_empty());
    assert!(
        request
            .corrections()
            .select(
                &source,
                WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN),
            )
            .is_err(),
        "the current disclosure query must actually include the original subject"
    );
}

fn independent_semantic_dependency() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    assert_eq!(
        request
            .corrections()
            .reverse(&selected)
            .inspect(history_limits())
            .unwrap(),
        WorthQueryCorrectionApplicability::ReadyForPreparation
    );
    host::edit(&host, Value::Ceiling(50), "semantic-dependency");
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 7
        }]
    );
    assert!(
        matches!(
            request
                .corrections()
                .reverse(&selected)
                .inspect(history_limits()),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::SourceDiverged
            ))
        ),
        "an unchanged after-image cannot hide a changed independent decision dependency"
    );
}

fn history_limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN)
}

fn reverse_after_disjoint_and_replay() {
    let host = host::install();
    let source = host::edit(&host, Value::Primary(11), "source");
    let parent = host::edit(&host, Value::Secondary(19), "disjoint");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let selected = request
        .corrections()
        .select(&source, history_limits())
        .unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-source")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("a fresh eligible reversal must prepare its inverse");
    };
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 11,
            secondary: 19
        }],
        "preparation must not publish the inverse"
    );
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("the prepared inverse must publish: {outcome:?}");
    };
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    assert_eq!(
        receipt.commit_reference().parents,
        vec![parent.commit_reference().commit_id]
    );
    let causality = receipt
        .aftermath_causality()
        .expect("the correction co-commits causal meaning");
    assert!(
        !receipt.has_retained_preimage(),
        "the inverse keeps its own irreversible contract"
    );
    assert!(matches!(request.corrections().select(&receipt, history_limits()),
        Err(WorthQueryApplicationCorrectionDenial::Source(
            worth_query_host::facade::primary_graph::WorthQueryCorrectionSelectionDenial::SourceNotCorrectable))),
        "retaining inverse input for replay cannot grant another reversal");
    assert_eq!(
        receipt.committed_product_publication().signal_posture(),
        worth_query_host::facade::product::CompositeComponentChangePosture::RetainExact
    );
    assert_eq!(
        receipt.committed_product_publication().signal_publication(),
        parent.committed_product_publication().signal_publication()
    );
    assert_eq!(
        receipt
            .committed_product_publication()
            .conditional_definition_generation(),
        parent
            .committed_product_publication()
            .conditional_definition_generation()
    );
    assert_eq!(causality.parent(), parent.commit_reference());
    let recorded = causality
        .recorded_source()
        .expect("the exact original source is retained as metadata");
    assert_eq!(recorded.relational(), source.commit_reference());
    assert_eq!(
        recorded.world(),
        source.committed_product_publication().composite_commit()
    );
    let retry = request
        .corrections()
        .reverse(&selected)
        .idempotency("reverse-source")
        .limits(history_limits())
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Settled(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
    ) = retry
    else {
        panic!("an exact retry must replay before after-image compatibility or inverse derivation");
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(
        host::read(&host),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    println!("recorded reversal and exact replay passed");
}

fn foreign_runtime_receipt_refuses() {
    let source_host = host::install();
    let source = host::edit(&source_host, Value::Primary(11), "source");
    let foreign = host::install();
    let scope = request_scope();
    let principal = authenticate_operator(foreign.installed_schema(), &scope);
    let request = foreign.runtime().request(&principal, &scope);
    assert!(matches!(request.corrections().select(&source, history_limits()),
        Err(WorthQueryApplicationCorrectionDenial::Source(
            worth_query_host::facade::primary_graph::WorthQueryCorrectionSelectionDenial::ForeignRuntime))),
        "an equal-looking foreign receipt cannot select this runtime's source");
    assert_eq!(
        host::read(&foreign),
        [Row {
            primary: 5,
            secondary: 7
        }]
    );
}
