//! Genuine independently governed observer roots inspect a nested source output.
use super::super::{
    adjacency_link, authentication::*, host, output_program::OutputProgram, program, schema::*,
};
use super::{capability, journey, SourceDisclosure};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, causal_inspection::*,
    declaration::application_program::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
pub mod grant_status;
pub mod lifecycle;
pub(super) mod observation;
pub mod query;
pub(super) mod seed;
pub fn connects_distinct_original_source_and_nested_query_subjects() {
    let application = install();
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    link(
        &application,
        seed::DECOY,
        seed::OBSERVER,
        "decoy-observer-link",
    );
    link(
        &application,
        seed::OBSERVER,
        "numeric",
        "observer-source-link",
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source, producer, read) = journey::perform(&application, 11, "distinct-source");
    let (after, row) = observation::read(&application, &principal, &scope, &read, seed::OBSERVER);
    assert_eq!(row.key, seed::OBSERVER);
    assert_eq!(row.references, vec![("numeric".to_owned(), 23)]);
    assert_ne!(row.key, row.references[0].0);
    let (decoy, decoy_row) =
        observation::read(&application, &principal, &scope, &read, seed::DECOY);
    assert_eq!(decoy_row.key, seed::DECOY);
    assert_eq!(decoy_row.references, vec![(seed::OBSERVER.to_owned(), 0)]);
    drop(read);
    let denial = inspect(
        &application,
        &principal,
        &scope,
        &decoy,
        &source,
        seed::DECOY,
    )
    .err()
    .expect("a real same-kind sibling is not the produced output");
    assert_eq!(
        denial.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::OutputUnconnected)
    );
    drop(denial);
    decoy.close();
    let report = inspect(
        &application,
        &principal,
        &scope,
        &after,
        &source,
        seed::OBSERVER,
    )
    .unwrap();
    assert_eq!(
        report.producer_connection().source_commit(),
        source.commit_reference()
    );
    assert_eq!(
        report.producer_connection().producer_commit(),
        producer.commit_reference()
    );
    assert_eq!(
        report.producer_connection().output_role(),
        "calculated-number"
    );
    report.close();
    assert_eq!(readers.observe().active(), baseline);
    after.close();
    assert_eq!(readers.observe().active(), baseline);
}
fn install() -> host::OutputHost {
    host::install_program_with_seed(
        program::validated::<OutputProgram>(),
        WorthQueryApplicationProgramRoster::new().support(program::validated::<program::Program>()),
        seed::seed,
    )
}
pub(super) fn link(application: &host::OutputHost, from: &str, to: &str, key: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let outcome = application
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(adjacency_link::Link {
            key: from.to_owned(),
            other: to.to_owned(),
            present: true,
            primary: 17,
        })
        .without_source()
        .idempotency(&key.to_owned())
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "actual authored reference: {outcome:?}"
    );
}
fn inspect<'o>(
    application: &host::OutputHost,
    principal: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    after: &'o WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
    query_subject: &str,
) -> Result<WorthQueryConnectedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial> {
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(after)
        .why_observed()
        .caused_by::<SourceDisclosure>(source)
        .reference_only()
        .execute_governed(
            access(application, principal, scope, query_subject),
            access(application, principal, scope, query_subject),
            access(application, principal, scope, "numeric"),
            access(application, principal, scope, "numeric"),
        )
}
fn access(
    application: &host::OutputHost,
    external: &WorthQueryAuthenticatedExternalPrincipal<NumericSchema>,
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> WorthQueryAdmittedApplicationCapabilityAccess<
    NumericSchema,
    capability::Inspect,
    capability::InspectCausalEvidence,
    capability::Input,
> {
    application
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.current_world())
                .select(&phase)
                .unwrap();
            let binding = application
                .installed_schema()
                .principal_binding(ActorBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    external,
                    scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let capability = application
                .installed_schema()
                .capability(
                    capability::Inspect::reference(),
                    capability::InspectCausalEvidence::reference(),
                )
                .unwrap();
            selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &capability,
                    capability::Input::at(subject),
                    scope,
                )
                .unwrap()
        })
        .unwrap()
}
