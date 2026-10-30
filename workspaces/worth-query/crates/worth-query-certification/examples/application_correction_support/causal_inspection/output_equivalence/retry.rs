//! A real request stops after the original Native pair is memoized in Delivered custody.
use super::super::super::{authentication::*, host, output_program::OutputProgram, query::full};
use super::super::{distinct_subjects::observation, journey as source_journey};
use super::{
    demand,
    journey::{inspect, install},
    program,
};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    primary_graph::*,
};

fn head(application: &host::OutputHost) -> impl Eq + std::fmt::Debug {
    application
        .with_host_advancement(|phase| {
            application
                .runtime()
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap()
}
pub fn completed_comparison_survives_late_cancellation_and_checkpoint_retry() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source2, _, read) =
        source_journey::perform(&application, 2, "comparison-retry-source-two");
    drop(read);
    let (_, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 1);
    let (original2, _) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    drop(read);
    let report2 = inspect(&application, &principal, &scope, &original2, &source2).unwrap();
    let native_revision = report2.producer_evaluation().observed_output_revision();
    report2.close();
    original2.close();

    let cancellation = WorthQueryCancellationSource::new();
    let rejected = WorthQueryCancellationSource::new();
    assert!(application
        .runtime()
        .cancel_after_next_output_comparison_for_test(cancellation.clone()));
    assert!(!application
        .runtime()
        .cancel_after_next_output_comparison_for_test(rejected.clone()));
    // The unrelated, non-opt-in arithmetic producer also completes readiness here.
    let (source4, _, read) =
        source_journey::perform(&application, 4, "comparison-retry-source-four");
    drop(read);
    assert!(
        !cancellation.token().is_cancelled(),
        "unrelated readiness cannot consume the comparison seam"
    );
    let cancelled_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let request = application.runtime().request(&principal, &cancelled_scope);
    let mut interrupted = request
        .demand(demand::Demand)
        .start_in_program::<OutputProgram, program::Root>(&application)
        .unwrap();
    let denial = interrupted
        .settle(&request)
        .err()
        .expect("actual late cancellation must refuse before Ready");
    let WorthQueryApplicationOutputDemandDenial::Demand(denial) = denial else {
        panic!("the checkpoint interruption must retain its actual demand cause: {denial:?}")
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Cancelled);
    assert!(cancellation.token().is_cancelled());
    assert!(
        !rejected.token().is_cancelled(),
        "a refused overwrite never replaced the original source"
    );
    drop(denial);
    assert_eq!(
        application
            .runtime()
            .output_checkpoint_snapshot_state_for_test()
            .1,
        0,
        "postpublication output records own no hidden snapshot"
    );
    let attempts_after_stop = application
        .runtime()
        .output_readiness_attempt_count_for_test();
    let committed_head = head(&application);
    let current = application
        .runtime()
        .request(&principal, &scope)
        .query(full::Read {
            key: "observer".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        current.rows()[0].calculated,
        0,
        "late optional failure cannot undo committed parity output"
    );
    drop(current);
    let (producer4, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 0, "Delivered retry contacts no handler");
    assert_eq!(
        head(&application),
        committed_head,
        "retry cannot publish another producer commit"
    );
    assert_eq!(
        application
            .runtime()
            .output_readiness_attempt_count_for_test()
            - attempts_after_stop,
        1,
        "retry may reevaluate readiness but must not allocate another baseline/candidate pair"
    );
    assert_eq!(
        application
            .runtime()
            .output_checkpoint_snapshot_state_for_test()
            .1,
        0
    );
    let (original4, row4) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    assert_eq!(row4.references, vec![("observer".to_owned(), 0)]);
    drop(read);
    let report4 = inspect(&application, &principal, &scope, &original4, &source4).unwrap();
    assert_eq!(
        report4.producer_evaluation().producer_commit(),
        producer4.commit_reference()
    );
    assert_eq!(
        report4.producer_evaluation().decision(),
        WorthQueryProducerEvaluationDecision::Equivalent
    );
    assert_eq!(
        report4.producer_evaluation().observed_output_revision(),
        native_revision
    );
    let (reopened, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 0);
    assert!(reopened.is_same_authoritative_commit(&producer4));
    let (original_reopened, _) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    drop(read);
    let reopened_report = inspect(
        &application,
        &principal,
        &scope,
        &original_reopened,
        &source4,
    )
    .unwrap();
    assert!(report4
        .producer_evaluation()
        .same_evaluation_as(reopened_report.producer_evaluation()));
    reopened_report.close();
    original_reopened.close();
    report4.close();
    original4.close();
    drop(interrupted);
    assert_eq!(readers.observe().active(), baseline);
}
