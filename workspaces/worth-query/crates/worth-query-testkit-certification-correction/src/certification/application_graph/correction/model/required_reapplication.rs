//! Retained source reapplication follows its original required-output completion owner.
use super::{
    authentication::*,
    correction::RecordedEdit,
    edit::{Edit, Value},
    host,
    output_program::{OutputProgram, Root},
    query::Row,
    required_output_courts::{head, limits, read_calculation, settle},
};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

mod ordinary;
mod overlapping;
pub fn ordinary_outputs() {
    ordinary::run(false);
    ordinary::run(true);
}
pub fn run() {
    run_in(host::install_with_outputs());
    run_in(host::install_with_output_limits(host::worker_limits()));
    overlapping::run(false);
    overlapping::run(true);
}
pub fn overlapping_fields() {
    overlapping::run(false);
    overlapping::run(true);
}
fn run_in(application: host::OutputHost) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let source_key = "required-reapplication-source".to_owned();
    let original =
        super::required_output_courts::co_committed::publish_and_deliver(&application, 23, || {
            request
                .advanced()
                .mutate(Edit {
                    key: "numeric".to_owned(),
                    value: Value::Primary(11),
                })
                .without_source()
                .idempotency(&source_key)
                .execute_performed::<OutputProgram, Root>(
                    &application,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
    assert_eq!(read_calculation(&application), 23);

    let selected = request.corrections().select(&original, limits()).unwrap();
    let reverse = request
        .corrections()
        .reverse(&selected)
        .idempotency("required-reapplication-reverse")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(reverse) = reverse else {
        panic!("reverse must prepare");
    };
    let reverse = reverse.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(reverse) = reverse else {
        panic!("reverse must perform");
    };
    let reversal = reverse.receipt().clone();
    settle(&application, reverse);
    assert_eq!(read_calculation(&application), 11);

    let disjoint_key = "required-reapplication-disjoint".to_owned();
    super::required_output_courts::co_committed::publish_and_deliver(&application, 11, || {
        request
            .advanced()
            .mutate(super::other_edit::OtherEdit {
                key: "numeric".to_owned(),
                value: Value::Secondary(19),
            })
            .without_source()
            .idempotency(&disjoint_key)
            .execute_performed::<OutputProgram, Root>(
                &application,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let page = request
        .corrections()
        .alternatives(&selected, limits(), NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    assert_eq!(page.entries().len(), 1);
    let chosen = page.entries()[0].clone();
    assert_eq!(chosen.performed_commit(), reversal.commit_reference());
    let alternative = page.select(&chosen, limits()).unwrap();
    assert!(matches!(
        request
            .corrections()
            .reapply(&alternative)
            .idempotency("required-reapplication-source-only")
            .limits(limits())
            .prepare(),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::RequiredOutputReconciliation
        ))
    ));

    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("required-reapplication")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("reapplication must prepare");
    };
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 5,
            secondary: 19
        }]
    );
    let performed = prepared.publish().unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = performed else {
        panic!("reapplication must retain output custody");
    };
    let reapplied = performed.receipt().clone();
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 11,
            secondary: 19
        }]
    );
    assert_eq!(
        read_calculation(&application),
        11,
        "source publication precedes required output delivery"
    );
    let fact = reapplied.aftermath_causality().unwrap();
    assert_eq!(fact.role(), WorthQueryAftermathCausalRole::RecordedReapply);
    assert_eq!(
        fact.recorded_source().relational(),
        original.commit_reference()
    );
    let lineage = fact.correction_lineage();
    assert_eq!(lineage.root().relational(), original.commit_reference());
    assert_eq!(
        lineage.selected_alternative().unwrap().relational(),
        reversal.commit_reference()
    );
    assert_eq!(
        lineage.active_reversal().unwrap().relational(),
        reversal.commit_reference()
    );
    settle(&application, performed);
    assert_eq!(read_calculation(&application), 23);
    assert_eq!(
        host::read(&application),
        [Row {
            operand: None,
            primary: 11,
            secondary: 19
        }]
    );

    let page = request
        .corrections()
        .alternatives(&selected, limits(), NonZeroUsize::MIN)
        .unwrap();
    let retry_alternative = page.select(&chosen, limits()).unwrap();
    let before_retry = head(&application);
    let retry = request
        .corrections()
        .reapply(&retry_alternative)
        .idempotency("required-reapplication")
        .limits(limits())
        .prepare_performed::<RecordedEdit, OutputProgram, Root>(&application)
        .unwrap();
    let WorthQueryRequiredCorrectionPreparation::Settled(
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
        ),
    ) = retry
    else {
        panic!("completed required reapplication must replay");
    };
    assert_eq!(replayed.commit_reference(), reapplied.commit_reference());
    assert_eq!(head(&application), before_retry);
    assert_eq!(read_calculation(&application), 23);
    println!("retained source reapplication preserves fixed required-output delivery and exact completed replay");
}
