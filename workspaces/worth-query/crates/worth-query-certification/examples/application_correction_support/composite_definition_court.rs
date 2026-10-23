//! Recorded correction publishes authored truth and its Signal definition together.
use super::{
    authentication::*, composite_definition_model::*, composite_definition_runtime::Conditional,
    edit, host, query::Row,
};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::*,
    runtime::{self, ExecutionAllocationPolicy},
};
#[path = "../../tests/support/execution_policy.rs"]
mod observation_request;

pub fn run() {
    let mut application = host::install();
    exercise(&application);
    application.close_conditional_runtime().unwrap();
    println!("recorded combined definition correction, stale target, replay and real conditional execution passed");
}

fn exercise(application: &host::Host) {
    let mut source = change(&application);
    assert_combined(&source);
    deliver(&application, &mut source);
    assert_eq!(
        host::read(&application),
        [Row {
            primary: 23,
            secondary: 7
        }]
    );
    assert_eq!(
        observe(
            &application,
            WorthQueryConditionalSignalDecision::Suppressed
        ),
        0,
        "the authored rejecting predicate suppresses the due wake"
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let limits = super::required_output_courts::limits();
    let selected = request.corrections().select(&source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("definition-stale")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("combined correction prepares");
    };
    host::edit(
        &application,
        edit::Value::Secondary(19),
        "definition-disjoint",
    );
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(&outcome, WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(cause)) if cause.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale),
        "prepared composite must reject the moved target: {outcome:?}"
    );
    assert_eq!(
        host::read(&application),
        [Row {
            primary: 23,
            secondary: 19
        }]
    );
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("definition-correct")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("fresh combined correction prepares");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { mut receipt, .. } = outcome else {
        panic!("combined correction publishes: {outcome:?}");
    };
    assert_combined(&receipt);
    assert_ne!(
        receipt.committed_product_publication().signal_publication(),
        source.committed_product_publication().signal_publication()
    );
    assert_eq!(
        receipt
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .unwrap()
            .relational(),
        source.commit_reference()
    );
    assert_eq!(
        host::read(&application),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    let corrected = super::required_output_courts::head(&application);
    let replay = request
        .corrections()
        .reverse(&selected)
        .idempotency("definition-correct")
        .limits(limits)
        .prepare()
        .unwrap();
    let replayed = match replay {
        WorthQueryApplicationCorrectionPreparation::Settled(
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(replayed),
        ) => replayed,
        WorthQueryApplicationCorrectionPreparation::Settled(other) => {
            panic!("exact replay must return the original combined receipt: {other:?}")
        }
        _ => panic!("exact replay must settle against the original combined publication"),
    };
    assert_eq!(replayed.commit_reference(), receipt.commit_reference());
    assert_eq!(
        replayed.committed_product_publication().composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        replayed
            .committed_product_publication()
            .signal_publication(),
        receipt.committed_product_publication().signal_publication()
    );
    assert_eq!(
        replayed
            .committed_product_publication()
            .conditional_definition_generation(),
        receipt
            .committed_product_publication()
            .conditional_definition_generation()
    );
    assert_eq!(
        replayed.aftermath_causality().unwrap().recorded_source(),
        receipt.aftermath_causality().unwrap().recorded_source()
    );
    assert_eq!(super::required_output_courts::head(&application), corrected);
    deliver(&application, &mut receipt);
    assert_eq!(
        observe(&application, WorthQueryConditionalSignalDecision::Eligible),
        1,
        "the corrected provider permits the actual due operation"
    );
    assert_eq!(
        host::read(&application)[0].secondary,
        99,
        "ordinary temporal re-entry committed its declared effect"
    );
}

pub(super) fn change(application: &host::Host) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let key = "definition-source".to_owned();
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Change {
            key: "numeric".to_owned(),
            value: 23,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("ordinary combined source publishes: {outcome:?}");
    };
    receipt
}
pub(super) fn assert_combined(receipt: &WorthQueryApplicationCommitReceipt) {
    let publication = receipt.committed_product_publication();
    assert_eq!(
        publication.relational_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication.signal_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert!(publication.signal_publication().is_some());
    assert!(publication.conditional_definition_generation().is_some());
}
pub(super) fn deliver(application: &host::Host, receipt: &mut WorthQueryApplicationCommitReceipt) {
    let change = receipt
        .take_performed_relational_product_change()
        .expect("the combined publication retains its real patch");
    let clock = application.runtime().conditional::<Conditional>().unwrap();
    application
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap();
            let delivered = selected
                .deliver_relational_change_to_conditional(&phase, &clock, 0, change)
                .unwrap();
            assert!(matches!(
                delivered,
                runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(_)
            ));
        })
        .unwrap();
}
pub(super) fn observe(
    application: &host::Host,
    expected: WorthQueryConditionalSignalDecision,
) -> usize {
    let clock = application.runtime().conditional::<Conditional>().unwrap();
    let mut port = application
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap();
            let execution = phase
                .checked_execution_request_for(application.runtime().product_runtime())
                .unwrap();
            selected.conditional_clock(&execution, &clock).unwrap()
        })
        .unwrap();
    let request = observation_request::serial_request();
    let observed = match port.observe(&worth_execution::ExecutionRequest::serial(&request)) {
        WorthQueryConditionalClockObservationOutcome::Accepted(observed) => observed,
        WorthQueryConditionalClockObservationOutcome::Failed(cause) => {
            panic!("conditional observation failed: {cause:?}")
        }
        _ => panic!("a fresh clock reading must be accepted"),
    };
    assert_eq!(observed.failed_operation_count(), 0);
    assert_eq!(observed.indeterminate_operation_count(), 0);
    assert_eq!(
        observed.due_wake_count(),
        1,
        "the authored intent must be due"
    );
    let [wake] = observed.execution_provenance() else {
        panic!(
            "one real wake must be evaluated: {:?}",
            observed.execution_provenance()
        );
    };
    assert_eq!(wake.signal_decision(), Some(expected), "{wake:?}");
    assert_eq!(observed.retained_failed_wake_count(), 0, "{wake:?}");
    observed.committed_operation_count()
}
