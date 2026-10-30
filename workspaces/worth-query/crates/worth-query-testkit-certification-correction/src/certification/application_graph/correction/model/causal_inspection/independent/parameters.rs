//! Genuine different query parameters cannot invent a projected output occurrence.
use super::*;
pub fn no_match_parameters_keep_original_output_evidence_unavailable() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (source, _, read) = journey::perform(&application, 11, "independent-parameter-source");
    let (after, rows) =
        retained_query::read_parameters(&application, &principal, &scope, &read, false);
    assert!(
        rows.is_empty(),
        "the genuine query predicate excludes the actual subject"
    );
    drop(read);
    let fresh = application.runtime().request(&principal, &scope);
    let denial = fresh
        .inspect::<output_query::Binding>(&after)
        .why_observed()
        .caused_by::<SourceDisclosure>(&source)
        .reference_only()
        .execute_governed(
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
            journey::access(&application, &principal, &scope),
        )
        .err()
        .expect("no observed output field exists for these original parameters");
    assert_eq!(
        denial.producer_connection_refusal(),
        Some(WorthQueryProducerConnectionDenial::OutputUnconnected)
    );
    after.close();
}
