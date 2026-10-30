//! An original produced field remains connected after its entity is retired.
use super::super::{authentication::*, host, program as source_program, query::full};
use super::{capability, distinct_subjects, journey, SourceDisclosure};
use distinct_subjects::{observation, query};
use worth_query_host::facade::{
    application_entry::*, application_installation::WorthQueryApplicationProgramRoster,
    causal_inspection::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
pub mod calculation;
mod demand;
mod lifecycle;
pub mod program;
pub mod readiness;

pub fn original_output_survives_retirement_and_equal_replacement() {
    let application = host::install_program_with_seed(
        source_program::validated::<super::super::output_program::OutputProgram>(),
        WorthQueryApplicationProgramRoster::new()
            .support(source_program::validated::<source_program::Program>()),
        distinct_subjects::seed::seed,
    );
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    distinct_subjects::link(&application, "numeric", "observer", "retained-output-link");
    let (source, _, source_read) = journey::perform(&application, 11, "retained-output-source");
    drop(source_read);
    let (producer, read) = demand::settle(&application);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let (after, row) = observation::read(&application, &principal, &scope, &read, "numeric");
    assert_eq!(row.key, "numeric");
    assert_eq!(row.references, vec![("observer".to_owned(), 23)]);
    let request = application.runtime().request(&principal, &scope);
    let original = request
        .query(full::Read {
            key: "observer".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .entity_id;
    drop(read);
    lifecycle::retire(&application);
    assert_eq!(
        row.reference_entities,
        vec![original],
        "the retained nested projection names the genuine original output"
    );
    match request
        .query(full::Read {
            key: "observer".to_owned(),
        })
        .execute()
    {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => assert_eq!(
            cause.kind(),
            WorthQueryEntityResolutionDenialKind::Refused(
                WorthQueryEntityResolutionRefusalKind::UnknownEntity
            )
        ),
        Err(other) => panic!("retired output read failed for another reason: {other:?}"),
        Ok(_) => panic!("the actual output must be absent after retirement"),
    }
    for replaced in [false, true] {
        if replaced {
            lifecycle::restore(&application, 23);
            let replacement = request
                .query(full::Read {
                    key: "observer".to_owned(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .entity_id;
            assert_ne!(
                original, replacement,
                "equal bytes and selector have a different full generation"
            );
            distinct_subjects::link(
                &application,
                "numeric",
                "observer",
                "replacement-output-link",
            );
            let head = request
                .query(query::Read {
                    key: "numeric".to_owned(),
                })
                .retain_projection_for_inspection()
                .execute_governed(
                    capability::Inspect::reference(),
                    capability::InspectCausalEvidence::reference(),
                    capability::Input::at("numeric"),
                )
                .unwrap();
            assert_eq!(
                head.rows()[0].references,
                row.references,
                "replacement is an equal-looking nested output"
            );
            assert_eq!(head.rows()[0].reference_entities, vec![replacement]);
            assert_ne!(head.rows()[0].reference_entities, row.reference_entities);
            let head = head.receipt().inspect().observation().unwrap().clone();
            let denied = inspect(&application, &principal, &scope, &head, &source)
                .err()
                .expect("a replacement head query cannot acquire the old supplier");
            assert_eq!(
                denied.producer_connection_refusal(),
                Some(WorthQueryProducerConnectionDenial::ProducerUnavailable)
            );
            head.close();
        }
        let report = inspect(&application, &principal, &scope, &after, &source).unwrap();
        let connection = report.producer_connection();
        assert_eq!(connection.source_commit(), source.commit_reference());
        assert_eq!(connection.producer_commit(), producer.commit_reference());
        assert_eq!(
            connection.output_publication_commit(),
            producer.commit_reference()
        );
        assert_eq!(connection.output_role(), "retained-number");
        report.close();
        assert_eq!(readers.observe().active(), baseline);
    }
    lifecycle::revoke_and_refuse(&application, &principal, &scope, &after, &source);
    assert_eq!(readers.observe().active(), baseline);
    after.close();
    assert_eq!(readers.observe().active(), baseline);
}

fn inspect<'o>(
    application: &host::OutputHost,
    principal: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<super::super::schema::NumericSchema>,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    after: &'o WorthQueryApplicationQueryObservation,
    source: &WorthQueryApplicationCommitReceipt,
) -> Result<WorthQueryConnectedApplicationInspection<'o>, WorthQueryApplicationInspectionDenial> {
    application
        .runtime()
        .request(principal, scope)
        .inspect::<query::Binding>(after)
        .why_observed()
        .caused_by::<SourceDisclosure>(source)
        .reference_only()
        .execute_governed(
            journey::access(application, principal, scope),
            journey::access(application, principal, scope),
            journey::access(application, principal, scope),
            journey::access(application, principal, scope),
        )
}
