//! Actual input-cutoff authority cannot become a performed output evaluation.
use super::super::super::{
    authentication::*,
    edit::Value,
    other_edit::OtherEdit,
    output_program::{OutputProgram, Root},
};
use super::super::{distinct_subjects::observation, journey as source_journey};
use super::{
    demand,
    journey::{inspect, install},
    retry::head,
};
use worth_query_host::facade::{
    application_entry::*, causal_inspection::*, primary_graph::WorthQueryOutputSettlementPosture,
};

pub fn stable_input_reuse_has_no_performed_output_evaluation() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source, _, read) =
        source_journey::perform(&application, 4, "stable-evaluation-source-four");
    drop(read);
    let (_, read, contacts) = demand::settle(&application);
    assert_eq!(contacts, 1);
    drop(read);
    let request = application.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".to_owned(),
            value: Value::Secondary(8),
        })
        .without_source()
        .idempotency(&"stable-evaluation-secondary-eight".to_owned())
        .execute_performed::<OutputProgram, Root>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the queried Secondary edit must retain its declared required output")
    };
    let (_, _, read) = source_journey::settle_source(&application, &request, performed, 1);
    drop(read);
    let row = application
        .runtime()
        .request(&principal, &scope)
        .query(super::super::super::query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!((row.rows()[0].primary, row.rows()[0].secondary), (4, 8));
    drop(row);
    let prior_head = head(&application);
    let (receipt, read, contacts, posture) = demand::settle_observation(&application);
    assert_eq!(posture, WorthQueryOutputSettlementPosture::StableReused);
    assert!(
        receipt.is_none(),
        "Stable authority is no performed receipt"
    );
    assert_eq!(contacts, 0);
    assert_eq!(
        head(&application),
        prior_head,
        "input cutoff publishes no new Native commit"
    );
    let (original, row) =
        observation::read(&application, &principal, &scope, &read, "observer-decoy");
    assert_eq!(row.references, vec![("observer".to_owned(), 0)]);
    drop(read);
    let availability = application
        .runtime()
        .request(&principal, &scope)
        .inspect::<super::super::distinct_subjects::query::Binding>(&original)
        .why_observed()
        .reference_only()
        .execute_governed(super::super::distinct_subjects::access(
            &application,
            &principal,
            &scope,
            "observer-decoy",
        ))
        .unwrap();
    assert_eq!(
        availability.evidence_availability().projection(),
        WorthQueryInspectionRetainedEvidence::Retained
    );
    assert_eq!(
        availability.evidence_availability().supplier(),
        WorthQueryInspectionSupplierEvidence::StableInputReuse
    );
    assert_eq!(
        availability.evidence_availability().native_observation(),
        WorthQueryInspectionRetainedEvidence::NotRetained
    );
    availability.close();
    let refusal = inspect(&application, &principal, &scope, &original, &source)
        .err()
        .expect("Stable input reuse cannot claim a performed comparison");
    assert_eq!(
        refusal.producer_evaluation_refusal(),
        Some(WorthQueryProducerEvaluationDenial::NotPerformed)
    );
    drop(refusal);
    original.close();
    assert_eq!(readers.observe().active(), baseline);
}
