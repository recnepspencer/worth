//! Independent expectations distinguish actual parity evaluations and retained original revisions.
use super::super::super::{
    authentication::*, host, output_program::OutputProgram, program as source_program,
};
use super::super::{
    distinct_subjects::{self, observation, query},
    journey as source_journey, SourceDisclosure,
};
use super::demand;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*,
    application_installation::WorthQueryApplicationProgramRoster, causal_inspection::*,
    primary_graph::*,
};

pub(super) fn install() -> host::OutputHost {
    let application = host::install_program_with_seed(
        source_program::validated::<OutputProgram>(),
        WorthQueryApplicationProgramRoster::new()
            .support(source_program::validated::<source_program::Program>()),
        distinct_subjects::seed::seed,
    );
    distinct_subjects::link(
        &application,
        "observer-decoy",
        "observer",
        "parity-viewer-output",
    );
    distinct_subjects::link(
        &application,
        "numeric",
        "observer-decoy",
        "parity-sibling-output",
    );
    application
}
pub(super) fn inspect<'o>(
    application: &host::OutputHost,
    principal: &WorthQueryAuthenticatedExternalPrincipal<
        super::super::super::schema::NumericSchema,
    >,
    scope: &WorthQueryRequestScope,
    original: &'o WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
) -> Result<WorthQueryEvaluatedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial> {
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(original)
        .why_observed()
        .evaluated_by::<SourceDisclosure>(source)
        .reference_only()
        .execute_governed(
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "observer-decoy"),
            distinct_subjects::access(application, principal, scope, "numeric"),
            distinct_subjects::access(application, principal, scope, "numeric"),
        )
}

pub fn parity_source_and_output_evaluation() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source2, _, source_read) = source_journey::perform(&application, 2, "parity-source-two");
    drop(source_read);
    let (producer2, read2, contacts2) = demand::settle(&application);
    assert_eq!(contacts2, 1, "the installed parity handler actually ran");
    let (original2, row2) =
        observation::read(&application, &principal, &scope, &read2, "observer-decoy");
    assert_eq!(row2.references, vec![("observer".to_owned(), 0)]);
    drop(read2);
    let report2 = inspect(&application,&principal,&scope,&original2,&source2).unwrap_or_else(|denial|
        panic!("real parity evaluation 2 must connect: {denial:?}; connection={:?}; evaluation={:?}",
            denial.producer_connection_refusal(),denial.producer_evaluation_refusal()));
    assert_eq!(
        report2.producer_evaluation().producer_commit(),
        producer2.commit_reference()
    );
    let original_revision = report2.producer_evaluation().observed_output_revision();

    let (source4, _, source_read) = source_journey::perform(&application, 4, "parity-source-four");
    drop(source_read);
    let (producer4, read4, contacts4) = demand::settle(&application);
    assert_eq!(contacts4, 1, "changed input cannot be zero-contact reuse");
    let (original4, row4) =
        observation::read(&application, &principal, &scope, &read4, "observer-decoy");
    let (sibling, sibling_row) =
        observation::read(&application, &principal, &scope, &read4, "numeric");
    assert_eq!(row4.references, vec![("observer".to_owned(), 0)]);
    assert_eq!(
        sibling_row.references,
        vec![("observer-decoy".to_owned(), 0)]
    );
    drop(read4);
    let report4 = inspect(&application,&principal,&scope,&original4,&source4).unwrap_or_else(|denial|
        panic!("real equivalent parity evaluation must connect: {denial:?}; connection={:?}; evaluation={:?}",
            denial.producer_connection_refusal(),denial.producer_evaluation_refusal()));
    let evaluation4 = report4.producer_evaluation();
    assert_eq!(
        evaluation4.decision(),
        WorthQueryProducerEvaluationDecision::Equivalent
    );
    assert_eq!(evaluation4.source_commit(), source4.commit_reference());
    assert_eq!(evaluation4.producer_commit(), producer4.commit_reference());
    assert_eq!(evaluation4.output_role(), "parity-number");
    assert_eq!(
        (
            evaluation4.output_field().entity(),
            evaluation4.output_field().aspect(),
            evaluation4.output_field().field()
        ),
        ("Number", "NumberFacts", "Calculated")
    );
    assert_eq!(
        evaluation4.observed_output_revision(),
        original_revision,
        "equivalence creates no Native field revision"
    );
    assert!(
        !evaluation4.same_evaluation_as(report2.producer_evaluation()),
        "source4 has its own actual comparison, not source2's retained success"
    );
    let wrong_source = inspect(&application, &principal, &scope, &original4, &source2)
        .err()
        .expect("old equal-output source is not the performed input");
    assert_eq!(
        wrong_source.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::SourceInputUnconnected)
    );
    drop(wrong_source);
    let request = application.runtime().request(&principal, &scope);
    let creation = request
        .inspect::<query::Binding>(&original4)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source4)
        .reference_only()
        .execute_governed(
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
            distinct_subjects::access(&application, &principal, &scope, "observer-decoy"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
        )
        .err()
        .expect("equivalence must not acquire the strict creation connection");
    assert_eq!(
        creation.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::OutputUnconnected)
    );
    drop(creation);
    let sibling_denial = request
        .inspect::<query::Binding>(&sibling)
        .why_observed()
        .evaluated_by::<SourceDisclosure>(&source4)
        .reference_only()
        .execute_governed(
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
            distinct_subjects::access(&application, &principal, &scope, "numeric"),
        )
        .err()
        .expect("equal bytes on a real same-kind sibling are not the evaluated field");
    assert_eq!(
        sibling_denial.producer_evaluation_refusal(),
        Some(WorthQueryProducerEvaluationDenial::ObservedFieldUnconnected)
    );
    drop(sibling_denial);
    sibling.close();

    let (reused, read_reused, contacts) = demand::settle(&application);
    assert_eq!(contacts, 0);
    assert_eq!(reused.commit_reference(), producer4.commit_reference());
    let (original_reused, _) = observation::read(
        &application,
        &principal,
        &scope,
        &read_reused,
        "observer-decoy",
    );
    drop(read_reused);
    let report_reused =
        inspect(&application, &principal, &scope, &original_reused, &source4).unwrap();
    assert!(
        report_reused
            .producer_evaluation()
            .same_evaluation_as(evaluation4),
        "reuse retains the original performed evaluation and issues no new comparison"
    );
    report_reused.close();
    original_reused.close();

    let (source5, _, source_read) = source_journey::perform(&application, 5, "parity-source-five");
    drop(source_read);
    let (producer5, read5, contacts5) = demand::settle(&application);
    assert_eq!(contacts5, 1);
    let (original5, row5) =
        observation::read(&application, &principal, &scope, &read5, "observer-decoy");
    assert_eq!(row5.references, vec![("observer".to_owned(), 1)]);
    drop(read5);
    let report5 = inspect(&application, &principal, &scope, &original5, &source5).unwrap();
    let evaluation5 = report5.producer_evaluation();
    assert_eq!(
        evaluation5.decision(),
        WorthQueryProducerEvaluationDecision::Changed
    );
    assert_eq!(evaluation5.producer_commit(), producer5.commit_reference());
    assert_eq!(
        evaluation5.observed_output_revision().version(),
        producer5.commit_reference().version_id
    );
    assert_ne!(evaluation5.observed_output_revision(), original_revision);
    assert!(!evaluation5.same_evaluation_as(evaluation4));
    report5.close();
    original5.close();
    report4.close();
    original4.close();
    report2.close();
    original2.close();
    assert_eq!(
        readers.observe().active(),
        baseline,
        "readonly evaluations retain no Native readers"
    );
}
