//! Authored source selection through the ordinary request and a real performed receipt.
#![recursion_limit = "256"]
mod application_correction_support;
// The support shares these modules with the application graph court, which
// loads them at the same paths; the court owns and lints them, and the
// example uses a part.
#[allow(dead_code, unused_imports)]
mod document_retention_model {
    #[path = "../../tests/application_graph/document_retention_model/operator_identity.rs"]
    pub mod operator_identity;
    pub mod host {
        #[path = "../../../tests/application_graph/document_retention_model/host/world_resources.rs"]
        pub(crate) mod world_resources;
    }
}
#[allow(dead_code)]
#[path = "../tests/support/execution_policy.rs"]
mod test_request_policy;
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
    let court = std::env::args().nth(1);
    let (done, completion) = std::sync::mpsc::channel();
    let worker = std::thread::Builder::new()
        .name("application-correction".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            match court.as_deref() {
                None => run(),
                Some("composite") => application_correction_support::composite_definition_court::run(),
                Some("composite-recovery") => application_correction_support::composite_definition_recovery::run(),
                Some("composite-unwind") => application_correction_support::composite_definition_recovery::unwind::run(),
                Some("authored") => application_correction_support::authored_identity_courts::run(),
                Some("race") => application_correction_support::correction_race_court::run(),
                Some("tree-controls") => application_correction_support::tree_controls::run(),
                Some("tree-history") => application_correction_support::history_reapplication::run(),
                Some("tree-outputs") => application_correction_support::required_reapplication::run(),
                Some("adoption-cycles") => application_correction_support::fork_head_adoption::run(),
                Some("tree") => {
                    application_correction_support::tree_reapplication::run();
                    application_correction_support::tree_controls::run();
                    application_correction_support::required_reapplication::run();
                    application_correction_support::history_reapplication::run();
                }
                Some("outputs") => {
                    application_correction_support::required_output_recovery::run();
                    application_correction_support::required_output_courts::run();
                }
                Some(other) => panic!("unknown correction court {other:?}; expected composite, composite-recovery, composite-unwind, authored, race, tree, tree-controls, tree-history, tree-outputs, adoption-cycles, outputs, or no argument for all"),
            }
            let _ = done.send(());
        })
        .expect("start the application thread");
    if let Err(std::sync::mpsc::RecvTimeoutError::Timeout) =
        completion.recv_timeout(std::time::Duration::from_secs(60))
    {
        panic!("the application correction runtime court stalled");
    }
    worker
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
}

fn run() {
    application_correction_support::tree_reapplication::run();
    application_correction_support::tree_controls::run();
    application_correction_support::required_reapplication::run();
    application_correction_support::history_reapplication::run();
    application_correction_support::correction_race_court::run();
    application_correction_support::authored_identity_courts::run();
    application_correction_support::composite_definition_court::run();
    application_correction_support::composite_definition_recovery::run();
    application_correction_support::composite_definition_recovery::unwind::run();
    application_correction_support::source_output_courts::run();
    application_correction_support::compensation_courts::run();
    application_correction_support::required_output_recovery::run();
    application_correction_support::required_output_courts::run();
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
    let recorded = causality.recorded_source();
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
